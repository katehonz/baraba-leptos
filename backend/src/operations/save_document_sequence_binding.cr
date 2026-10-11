class SaveDocumentSequenceBinding < DocumentSequenceBinding::SaveOperation
  permit_columns company_id, family, document_type, document_number_sequence_id
end
