class SaveFixedAssetTransaction < FixedAssetTransaction::SaveOperation
  permit_columns transaction_date, amount, movement_type, description,
    document_number, document_date, fixed_asset_id, company_id

  before_save do
    validate_required transaction_date, amount
  end
end
