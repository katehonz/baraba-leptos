class SaveFixedAssetCategory < FixedAssetCategory::SaveOperation
  permit_columns name, description, cita_category, min_depreciation_rate, max_depreciation_rate,
    default_method, company_id, asset_account_id, depreciation_account_id, expense_account_id

  before_save do
    validate_required name, min_depreciation_rate, max_depreciation_rate
    validate_inclusion_of default_method, in: ["straight_line", "declining_balance"]
    validate_inclusion_of cita_category, in: ["I", "II", "III", "IV", "V", "VI", "VII"], allow_nil: true
  end
end
