class SaveInvoiceLine < InvoiceLine::SaveOperation
  permit_columns description, quantity, unit_price, vat_rate, line_total, invoice_id, account_id

  before_save do
    validate_required description, quantity, unit_price

    # Auto-calculate line_total if not provided
    if line_total.value.nil? && quantity.value && unit_price.value
      q = quantity.value.not_nil!
      p = unit_price.value.not_nil!
      line_total.value = q * p
    end
  end
end
