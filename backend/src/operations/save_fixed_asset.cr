class SaveFixedAsset < FixedAsset::SaveOperation
  permit_columns inventory_number, name, description, acquisition_date, put_into_service_date,
    acquisition_cost, residual_value, accounting_book_value, depreciation_method,
    accounting_depreciation_rate, tax_depreciation_rate, status,
    document_number, document_date, company_id, category_id,
    sap_accumulated_depreciation

  before_save do
    validate_required inventory_number, name, acquisition_date, acquisition_cost,
      depreciation_method, accounting_depreciation_rate, tax_depreciation_rate
    validate_inclusion_of depreciation_method, in: ["straight_line", "declining_balance"]
    validate_inclusion_of status, in: ["active", "disposed", "fully_depreciated"]
  end
end
