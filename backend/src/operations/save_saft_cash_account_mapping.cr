class SaveSaftCashAccountMapping < SaftCashAccountMapping::SaveOperation
  permit_columns company_id, cash_movement_type, debit_account, credit_account,
    debit_analytical, credit_analytical, description, is_active

  before_save do
    validate_required cash_movement_type
    validate_required debit_account
    validate_required credit_account
  end
end
