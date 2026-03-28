class SaveScannedInvoice < ScannedInvoice::SaveOperation
  permit_columns direction, status, vendor_name, vendor_vat_number, vendor_address,
    customer_name, customer_vat_number, customer_address,
    invoice_number, invoice_date, due_date,
    subtotal, total_tax, invoice_total,
    vies_status, vies_validation_message, vies_company_name, vies_company_address, vies_validated_at,
    counterparty_account_id, vat_account_id, expense_revenue_account_id,
    requires_manual_review, manual_review_reason, notes,
    confidence, original_file_name, s3_key, s3_key_json, azure_raw_json, vat_period,
    company_id, journal_entry_id, counterpart_id

  before_save do
    validate_required direction
    validate_inclusion_of direction, in: ["purchase", "sale"]
    validate_inclusion_of status, in: ["pending", "validated", "rejected", "processed"]
    validate_inclusion_of vies_status, in: ["pending", "valid", "invalid", "not_applicable", "error"]
  end
end
