class SaveOpeningBalance < OpeningBalance::SaveOperation
  permit_columns debit, credit, date, quantity, description, balance_type,
                 account_id, counterpart_id, product_id, warehouse_id, accounting_period_id

  needs set_company_id : Int64

  before_save do
    company_id.value = set_company_id
  end
end
