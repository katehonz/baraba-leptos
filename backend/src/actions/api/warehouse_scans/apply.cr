class Api::WarehouseScans::Apply < ApiAction
  include Api::InvoiceParams
  include Api::WarehouseScanJson

  post "/api/companies/:company_id/warehouse_scans/:warehouse_scan_id/apply" do
    cid = company_id.to_i64
    authorize!(Permission::WAREHOUSE_CREATE, cid)
    company = CompanyQuery.new.find(cid)
    found = WarehouseScanQuery.new.company_id(cid).id(warehouse_scan_id).first?
    if found.nil?
      response.status_code = 404
      return json({success: false, error: "Не е намерен"})
    end
    scan = found.not_nil!

    data = nested_or_flat(params.from_json, "apply")
    create_invoice = json_bool(data["create_invoice"]?) || false
    create_orphan = json_bool(data["create_orphan_receipt"]?) || false
    location_id = json_i64(data["location_id"]?)

    begin
      result = WarehouseScanApplier.apply!(scan, company, create_invoice, create_orphan, location_id)
      scan = WarehouseScanQuery.new.id(scan.id).first.not_nil!
      json({
        success: true,
        data:    scan_json(scan).merge({
          invoice_id:            result[:invoice].try(&.id),
          warehouse_document_id: result[:warehouse_document].try(&.id),
        }),
      })
    rescue ex : WarehouseScanApplier::Error
      response.status_code = 422
      json({success: false, error: ex.message})
    end
  end
end
