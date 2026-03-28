class SaveSaftMovementAccountMapping < SaftMovementAccountMapping::SaveOperation
  permit_columns(
    company_id,
    movement_type_code,
    debit_account,
    credit_account,
    debit_analytical,
    credit_analytical,
    description,
    is_active
  )
end
