class Api::WarehouseScans::Lookup < ApiAction
  include Api::InvoiceParams

  post "/api/companies/:company_id/warehouse_scans/lookup" do
    cid = company_id.to_i64
    authorize!(Permission::WAREHOUSE_READ, cid)
    data = nested_or_flat(params.from_json, "scan")
    sku = json_str(data["vendor_sku"]?)
    desc = json_str(data["vendor_description"]?)
    barcode = json_str(data["vendor_barcode"]?)
    cp = json_i64(data["counterpart_id"]?)
    mapping = VendorProductMappingService.find(cid, cp, sku, barcode, desc)
    json({
      success: true,
      data:    mapping ? {
        product_id: mapping.product_id,
        mapping_id: mapping.id,
        confirmed:  mapping.confirmed,
        hit_count:  mapping.hit_count,
      } : nil,
    })
  end
end
