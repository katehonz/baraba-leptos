class Api::WarehouseScans::UpdateLine < ApiAction
  include Api::InvoiceParams
  include Api::WarehouseScanJson

  put "/api/companies/:company_id/warehouse_scans/:warehouse_scan_id/lines/:line_id" do
    cid = company_id.to_i64
    authorize!(Permission::WAREHOUSE_UPDATE, cid)
    found_scan = WarehouseScanQuery.new.company_id(cid).id(warehouse_scan_id).first?
    if found_scan.nil?
      response.status_code = 404
      return json({success: false, error: "Не е намерен"})
    end
    scan = found_scan.not_nil!
    if scan.status == "applied"
      response.status_code = 422
      return json({success: false, error: "Приложен скан не се редактира"})
    end

    found_line = WarehouseScanLineQuery.new.id(line_id).warehouse_scan_id(scan.id).first?
    if found_line.nil?
      response.status_code = 404
      return json({success: false, error: "Редът не е намерен"})
    end
    line = found_line.not_nil!

    data = nested_or_flat(params.from_json, "line")
    product_id = json_i64(data["product_id"]?)

    if has_key?(data, "create_product") && data["create_product"]?
      created = create_product_from(cid, data["create_product"], line)
      unless created
        response.status_code = 422
        return json({success: false, error: "Неуспешно създаване на артикул"})
      end
      product_id = created.id
    end

    mapping_status = line.mapping_status
    new_product_id = line.product_id
    if product_id
      mapping_status = "manual"
      new_product_id = product_id
      VendorProductMappingService.remember!(
        cid,
        product_id.not_nil!,
        scan.counterpart_id,
        scan.vendor_name,
        line.raw_sku,
        line.raw_name,
        line.raw_barcode,
        line.unit_code,
        true
      )
    elsif has_key?(data, "product_id")
      mapping_status = "unmapped"
      new_product_id = nil
    end

    SaveWarehouseScanLine.update!(line,
      product_id: new_product_id,
      mapping_status: mapping_status,
      quantity: json_f64(data["quantity"]?) || line.quantity,
      unit_price: json_f64(data["unit_price"]?) || line.unit_price,
      vat_rate: json_f64(data["vat_rate"]?) || line.vat_rate,
      unit_code: json_str(data["unit_code"]?) || line.unit_code
    )

    scan = refresh_scan_status(scan)
    json({success: true, data: scan_json(scan)})
  end

  private def create_product_from(cid : Int64, node : JSON::Any, line : WarehouseScanLine) : Product?
    code = json_str(node["code"]?) || line.raw_sku.presence || "WS#{line.id}"
    name = json_str(node["name"]?) || line.raw_name.presence || code
    unit = json_str(node["unit_code"]?) || line.unit_code
    price = json_f64(node["price"]?) || line.unit_price
    SaveProduct.create!(
      company_id: cid,
      code: code,
      name: name,
      price: price,
      product_type: "30",
      goods_services_id: "01",
      uom_standard: unit,
      measure_unit: unit
    )
  rescue
    nil
  end
end
