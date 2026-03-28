class SaveBankTransaction < BankTransaction::SaveOperation
  permit_columns date, amount, currency, amount_base, exchange_rate, description,
    contra_account, contra_name, reference,
    salt_edge_transaction_id, salt_edge_category, wise_transaction_id,
    bank_account_id, journal_entry_id

  before_save do
    validate_required date, amount
  end
end
