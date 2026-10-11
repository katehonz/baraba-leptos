class SaveWarehouseScanLine < WarehouseScanLine::SaveOperation
  permit_columns warehouse_scan_id, raw_name, raw_sku, raw_barcode, quantity, raw_unit,
    unit_code, unit_price, vat_rate, mapping_status, confidence, line_number, product_id
end
