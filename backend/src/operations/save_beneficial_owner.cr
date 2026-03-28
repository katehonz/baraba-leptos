class SaveBeneficialOwner < BeneficialOwner::SaveOperation
  permit_columns first_name_bg, last_name_bg, egn,
    first_name_latin, last_name_latin, country,
    ownership_percentage, is_active, company_id

  before_save do
    validate_required first_name_bg
    validate_required last_name_bg
  end
end
