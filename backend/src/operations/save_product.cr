class SaveProduct < Product::SaveOperation
  permit_columns code, name, description, product_type, price, measure_unit,
    commodity_code, tax_type, tax_code, is_active,
    company_id, inventory_account_id, revenue_account_id

  before_save do
    validate_required code, name
    validate_inclusion_of product_type, in: ["10", "20", "30", "40", "50"], allow_nil: true
  end
end
