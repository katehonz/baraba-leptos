class SaveRole < Role::SaveOperation
  permit_columns name, description, permissions

  before_save do
    validate_required name
  end
end
