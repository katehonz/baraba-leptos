class SaveControlisyImport < ControlisyImport::SaveOperation
  permit_columns filename, import_type, status, documents_count, contractors_count,
    journal_entries_count, error_message, imported_at, company_id

  before_save do
    validate_required filename
    validate_required import_type
    validate_inclusion_of import_type, in: ["purchases", "sales", "bank"]
    validate_inclusion_of status, in: ["pending", "processing", "completed", "failed"]
  end
end
