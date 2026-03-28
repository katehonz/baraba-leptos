class SaveInvoice < Invoice::SaveOperation
  permit_columns invoice_number, issue_date, due_date, subtotal, vat_amount,
     total_amount, currency, status, notes, company_id, counterpart_id, document_type, exchange_rate, payment_method, tax_event_date, vat_exemption_reason, has_inventory

  before_save do
    validate_required invoice_number, issue_date, subtotal, vat_amount, total_amount
    validate_inclusion_of status, in: ["draft", "posted", "cancelled"]
    validate_inclusion_of currency, in: ["BGN", "EUR", "USD", "GBP"]
  end
end
