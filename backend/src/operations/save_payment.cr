class SavePayment < Payment::SaveOperation
  permit_columns payment_ref_no, period, period_year, transaction_id, transaction_date,
                 payment_method, description, batch_id, system_id, source_id,
                 saft_customer_id, saft_supplier_id, net_total, gross_total,
                 currency_code, exchange_rate,
                 company_id, counterpart_id, journal_entry_id, bank_account_id

  before_save do
    validate_required payment_ref_no, message: "is required"
    validate_required transaction_id, message: "is required"
    validate_required transaction_date, message: "is required"
    validate_required payment_method, message: "is required"
    validate_required description, message: "is required"
    validate_required gross_total, message: "is required"

    # Validate payment method
    if pm = payment_method.value
      unless ["01", "02", "03"].includes?(pm)
        payment_method.add_error "must be 01, 02, or 03"
      end
    end
  end
end
