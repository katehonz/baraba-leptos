class SaveProduct < Product::SaveOperation
  permit_columns code, name, description, product_type, price, measure_unit,
    commodity_code, tax_type, tax_code, is_active,
    company_id, inventory_account_id, revenue_account_id, expense_account_id,
    goods_services_id, product_group, product_number_code, valuation_method,
    uom_base, uom_standard, uom_conversion_factor

  before_save do
    validate_required code, name
    validate_inclusion_of product_type, in: ["10", "20", "30", "40", "50"], allow_nil: true
    validate_inclusion_of goods_services_id, in: ["01", "02"], allow_nil: true
    if product_type.value == "S"
      product_type.add_error("използвайте goods_services_id=02 за услуги")
    end
    if goods_services_id.value == "02"
      product_type.value = nil
    end
    uom = uom_standard.value.presence || measure_unit.value.presence || "C62"
    uom_standard.value = uom
    measure_unit.value = measure_unit.value.presence || uom
    unless SaftUnitOfMeasureQuery.new.code(uom).first?
      uom_standard.add_error("неизвестна мерна единица")
    end
  end
end

