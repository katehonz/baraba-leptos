class SaveSaftAccount < SaftAccount::SaveOperation
  permit_columns code, name, section_code, section_name, group_code, group_name

  before_save do
    validate_required code
    validate_required name
    validate_uniqueness_of code
  end
end
