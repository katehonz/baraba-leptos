class SaveVatRate < VatRate::SaveOperation
  permit_columns name, percentage, code, saft_tax_type, saft_tax_code,
    effective_from, effective_to, is_active

  before_save do
    validate_required name, code
  end
end
