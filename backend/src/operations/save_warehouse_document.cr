class SaveWarehouseDocument < WarehouseDocument::SaveOperation
  permit_columns company_id, document_type, document_number, issue_date, status,
    notes, source_id, movement_type, sequence_id, from_location_id, to_location_id,
    journal_entry_id, invoice_id, original_warehouse_document_id
end
