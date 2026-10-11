class SaveDocumentNumberSequence < DocumentNumberSequence::SaveOperation
  permit_columns company_id, code, name, current_number, padding, is_tax, reset_policy
end
