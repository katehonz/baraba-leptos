class SaveAssetTransaction < AssetTransaction::SaveOperation
  permit_columns asset_transaction_id, asset_transaction_type, description,
                 transaction_date, transaction_id, saft_customer_id, saft_supplier_id,
                 valuation_type, acquisition_cost_on_transaction,
                 book_value_on_transaction, transaction_amount,
                 company_id, fixed_asset_id, counterpart_id, journal_entry_id

  before_save do
    validate_required asset_transaction_id, message: "is required"
    validate_required asset_transaction_type, message: "is required"
    validate_required transaction_date, message: "is required"
    validate_required book_value_on_transaction, message: "is required"
    validate_required transaction_amount, message: "is required"
  end
end
