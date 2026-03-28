class SaveBankAccount < BankAccount::SaveOperation
  permit_columns name, iban, bic, currency, bank_name, integration_type,
    company_id, gl_account_id, buffer_account_id,
    salt_edge_connection_id, salt_edge_account_id, salt_edge_provider_code,
    salt_edge_last_sync, is_salt_edge_connected,
    wise_account_id, wise_balance_type, wise_last_sync, is_wise_connected

  before_save do
    validate_required name, iban, currency
    validate_inclusion_of integration_type, in: ["manual", "salt_edge", "wise"]
  end
end
