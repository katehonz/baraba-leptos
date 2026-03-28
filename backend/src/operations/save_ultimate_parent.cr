class SaveUltimateParent < UltimateParent::SaveOperation
  permit_columns name_bg, name_latin, uic, country, is_active, company_id

  before_save do
    validate_required name_bg
  end
end
