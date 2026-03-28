class SaveStockTransaction < StockTransaction::SaveOperation
  permit_columns transaction_date, quantity, unit_price, total_amount, is_incoming,
    movement_type, document_number, document_date,
    product_id, company_id, warehouse_id, journal_entry_id

  before_save do
    validate_required transaction_date, quantity, unit_price, is_incoming

    # Calculate total amount if not provided
    if total_amount.value.nil? && quantity.value && unit_price.value
      total_amount.value = quantity.value.not_nil! * unit_price.value.not_nil!
    end
  end
end
