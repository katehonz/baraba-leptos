class Api::WarehouseScans::Index < ApiAction
  include Api::WarehouseScanJson

  get "/api/companies/:company_id/warehouse_scans" do
    cid = company_id.to_i64
    authorize!(Permission::WAREHOUSE_READ, cid)
    scans = WarehouseScanQuery.new.company_id(cid)
    if status = params.get?("status")
      scans = scans.status(status.to_s)
    end
    json({
      success: true,
      data:    scans.map { |s| scan_json(s) },
    })
  end
end
