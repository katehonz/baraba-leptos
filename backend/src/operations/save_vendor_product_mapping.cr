class SaveVendorProductMapping < VendorProductMapping::SaveOperation
  permit_columns company_id, counterpart_id, vendor_name, vendor_sku, vendor_description,
    product_id, vendor_barcode, vendor_name_normalized, vendor_unit_code,
    conversion_factor, confirmed, hit_count, last_seen_at
end
