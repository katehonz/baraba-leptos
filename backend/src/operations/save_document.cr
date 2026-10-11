class SaveDocument < Document::SaveOperation
  permit_columns title, description, file_url, file_type, file_size, status,
    company_id, created_by_id

  before_save do
    validate_required title
    validate_inclusion_of status, in: ["pending", "processed", "archived"]
  end
end
