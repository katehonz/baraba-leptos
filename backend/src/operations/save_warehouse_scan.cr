class SaveWarehouseScan < WarehouseScan::SaveOperation
  permit_columns company_id, direction, status, original_file_name, s3_key, raw_json,
    vendor_name, vendor_vat, invoice_number, document_date, subtotal, vat_amount,
    total_amount, confidence, notes, counterpart_id, warehouse_document_id, invoice_id
end
