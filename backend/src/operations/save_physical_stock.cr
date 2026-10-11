class SavePhysicalStock < PhysicalStock::SaveOperation
  permit_columns location_id, stock_account_no, product_type, product_status,
                 stock_commodity_code, owner_id, uom_physical_stock, uom_conversion_factor,
                 unit_price_begin, unit_price_end, opening_stock_quantity, opening_stock_value,
                 closing_stock_quantity, closing_stock_value, period_start, period_end,
                 characteristics, company_id, warehouse_id, product_id

  before_save do
    validate_required product_type, message: "is required"
    validate_required owner_id, message: "is required"
    validate_required uom_physical_stock, message: "is required"

    # Validate product_type
    if pt = product_type.value
      unless ["10", "20", "30", "40", "50"].includes?(pt)
        product_type.add_error "must be 10, 20, 30, 40, or 50"
      end
    end
  end
end
