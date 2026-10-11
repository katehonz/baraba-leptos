class Api::WarehouseScans::Remember < ApiAction
  include Api::InvoiceParams

  post "/api/companies/:company_id/warehouse_scans/remember" do
    cid = company_id.to_i64
    authorize!(Permission::WAREHOUSE_UPDATE, cid)
    data = nested_or_flat(params.from_json, "mapping")
    pid = json_i64(data["product_id"]?)
    unless pid
      response.status_code = 422
      return json({success: false, error: "product_id required"})
    end
    m = VendorProductMappingService.remember!(
      cid,
      pid,
      json_i64(data["counterpart_id"]?),
      json_str(data["vendor_name"]?),
      json_str(data["vendor_sku"]?),
      json_str(data["vendor_description"]?),
      json_str(data["vendor_barcode"]?),
      json_str(data["vendor_unit_code"]?),
      true
    )
    json({success: true, data: {id: m.id, product_id: m.product_id}})
  end
end
