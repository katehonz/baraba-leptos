class SaveWarehouseDocumentLine < WarehouseDocumentLine::SaveOperation
  permit_columns warehouse_document_id, product_id, quantity, unit_code, unit_price, line_total, line_number
end
