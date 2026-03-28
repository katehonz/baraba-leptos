class SaveJournalEntry < JournalEntry::SaveOperation
  permit_columns entry_date, description, reference, status, company_id,
    document_number, document_date, vat_period, vat_purchase_operation,
    vat_sales_operation, total_amount, total_vat_amount, payment_method_code, lines,
    counterpart_id, branch_code

  before_save do
    validate_required entry_date, description
    validate_inclusion_of status, in: ["draft", "posted", "reversed"]
    generate_transaction_id if transaction_id.value.nil? || transaction_id.value.try(&.empty?)
  end

  private def generate_transaction_id
    ts = Time.utc.to_unix
    rnd = Random.rand(9999).to_s.rjust(4, '0')
    transaction_id.value = "JE-#{ts}-#{rnd}"
  end
end
