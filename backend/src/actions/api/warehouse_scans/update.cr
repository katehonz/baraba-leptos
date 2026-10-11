class Api::WarehouseScans::Update < ApiAction
  include Api::InvoiceParams
  include Api::WarehouseScanJson

  put "/api/companies/:company_id/warehouse_scans/:warehouse_scan_id" do
    cid = company_id.to_i64
    authorize!(Permission::WAREHOUSE_UPDATE, cid)
    found = WarehouseScanQuery.new.company_id(cid).id(warehouse_scan_id).first?
    if found.nil?
      response.status_code = 404
      return json({success: false, error: "Не е намерен"})
    end
    scan = found.not_nil!
    if scan.status == "applied"
      response.status_code = 422
      return json({success: false, error: "Приложен скан не се редактира"})
    end

    data = nested_or_flat(params.from_json, "scan")
    status = pick_str(data, "status", scan.status) || scan.status
    unless WarehouseScan::STATUSES.includes?(status)
      response.status_code = 422
      return json({success: false, error: "Невалиден статус"})
    end

    previous_cp = scan.counterpart_id
    updated = SaveWarehouseScan.update!(scan,
      counterpart_id: pick_i64(data, "counterpart_id", scan.counterpart_id),
      notes: pick_str(data, "notes", scan.notes),
      status: status,
      invoice_number: pick_str(data, "invoice_number", scan.invoice_number),
      vendor_name: pick_str(data, "vendor_name", scan.vendor_name)
    )
    if updated.counterpart_id != previous_cp
      VendorProductMappingService.apply_to_unmapped_scan_lines!(cid, updated)
      updated = refresh_scan_status(updated)
    end
    json({success: true, data: scan_json(updated)})
  end
end
