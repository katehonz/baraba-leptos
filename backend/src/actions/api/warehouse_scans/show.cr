class Api::WarehouseScans::Show < ApiAction
  include Api::WarehouseScanJson

  get "/api/companies/:company_id/warehouse_scans/:warehouse_scan_id" do
    cid = company_id.to_i64
    authorize!(Permission::WAREHOUSE_READ, cid)
    found = WarehouseScanQuery.new.company_id(cid).id(warehouse_scan_id).first?
    if found.nil?
      response.status_code = 404
      return json({success: false, error: "Не е намерен"})
    end
    json({success: true, data: scan_json(found.not_nil!)})
  end
end
