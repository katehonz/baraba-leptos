module VendorProductMappingService
  extend self

  def normalize(text : String?) : String?
    return nil if text.nil?
    s = text.downcase.gsub(/[[:punct:]]/, " ").gsub(/\s+/, " ").strip
    s.presence
  end

  def find(company_id : Int64, counterpart_id : Int64?, sku : String?, barcode : String?, name : String?) : VendorProductMapping?
    if counterpart_id && (bc = barcode.try(&.presence))
      if m = scoped(company_id, counterpart_id).vendor_barcode(bc).first?
        return m
      end
    end
    if counterpart_id && (s = sku.try(&.presence))
      if m = scoped(company_id, counterpart_id).vendor_sku(s).first?
        return m
      end
    end
    if counterpart_id && (n = normalize(name))
      if m = scoped(company_id, counterpart_id).vendor_name_normalized(n).first?
        return m
      end
    end
    if (bc = barcode.try(&.presence))
      if m = VendorProductMappingQuery.new.company_id(company_id).vendor_barcode(bc).first?
        return m
      end
    end
    if (s = sku.try(&.presence))
      if m = VendorProductMappingQuery.new.company_id(company_id).vendor_sku(s).first?
        return m
      end
    end
    if (n = normalize(name))
      if m = VendorProductMappingQuery.new.company_id(company_id).vendor_name_normalized(n).first?
        return m
      end
    end
    if (d = name.try(&.presence))
      VendorProductMappingQuery.new.company_id(company_id).vendor_description(d).first?
    end
  end

  def auto_product_id(mapping : VendorProductMapping?, confidence : Float64?) : Int64?
    return nil unless mapping
    conf = confidence || 0.0
    if mapping.confirmed || (mapping.hit_count >= 2 && conf > 0.85)
      mapping.product_id
    else
      nil
    end
  end

  def remember!(
    company_id : Int64,
    product_id : Int64,
    counterpart_id : Int64?,
    vendor_name : String?,
    vendor_sku : String?,
    vendor_description : String?,
    vendor_barcode : String? = nil,
    vendor_unit_code : String? = nil,
    confirmed : Bool = true
  ) : VendorProductMapping
    existing = find(company_id, counterpart_id, vendor_sku, vendor_barcode, vendor_description || vendor_name)
    hits = existing ? existing.hit_count + 1 : 1
    attrs = {
      company_id:             company_id,
      product_id:             product_id,
      counterpart_id:         counterpart_id,
      vendor_name:            vendor_name,
      vendor_sku:             vendor_sku,
      vendor_description:     vendor_description,
      vendor_barcode:         vendor_barcode,
      vendor_name_normalized: normalize(vendor_description || vendor_name),
      vendor_unit_code:       vendor_unit_code,
      confirmed:              confirmed,
      hit_count:              hits,
      last_seen_at:           Time.utc,
    }
    if existing
      SaveVendorProductMapping.update!(existing, **attrs)
    else
      SaveVendorProductMapping.create!(**attrs)
    end
  end

  def touch!(mapping : VendorProductMapping) : VendorProductMapping
    SaveVendorProductMapping.update!(mapping,
      hit_count: mapping.hit_count + 1,
      last_seen_at: Time.utc
    )
  end

  # Fill unmapped scan lines from remembered vendor → our-product maps.
  # Manual mappings are left untouched.
  def apply_to_unmapped_scan_lines!(company_id : Int64, scan : WarehouseScan) : Int32
    n = 0
    WarehouseScanLineQuery.new.warehouse_scan_id(scan.id).mapping_status("unmapped").each do |line|
      mapping = find(company_id, scan.counterpart_id, line.raw_sku, line.raw_barcode, line.raw_name)
      pid = auto_product_id(mapping, line.confidence || scan.confidence)
      next unless pid
      touch!(mapping.not_nil!) if mapping
      SaveWarehouseScanLine.update!(line, product_id: pid, mapping_status: "auto")
      n += 1
    end
    n
  end

  private def scoped(company_id : Int64, counterpart_id : Int64)
    VendorProductMappingQuery.new.company_id(company_id).counterpart_id(counterpart_id)
  end
end
