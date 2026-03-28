class SaveSaftAssetAccountMapping < SaftAssetAccountMapping::SaveOperation
  permit_columns company_id, asset_transaction_type, debit_account, credit_account,
                 debit_analytical, credit_analytical, description, is_active
end
