class SavePaymentLine < PaymentLine::SaveOperation
  permit_columns line_number, source_document_id, description, debit_credit_indicator,
                 tax_point_date, amount, currency_code, currency_amount, exchange_rate,
                 saft_customer_id, saft_supplier_id, tax_type, tax_code,
                 tax_percentage, tax_base, tax_amount,
                 payment_id, account_id, invoice_id

  before_save do
    validate_required debit_credit_indicator, message: "is required"
    validate_required amount, message: "is required"

    # Validate debit_credit_indicator
    if dci = debit_credit_indicator.value
      unless ["D", "C"].includes?(dci)
        debit_credit_indicator.add_error "must be D or C"
      end
    end
  end
end
