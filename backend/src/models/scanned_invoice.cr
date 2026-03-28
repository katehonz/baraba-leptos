# Scanned Invoice - сканирана фактура за OCR обработка
class ScannedInvoice < BaseModel
  table do
    column direction : String           # purchase, sale
    column status : String = "pending"  # pending, validated, rejected, processed

    # Vendor info
    column vendor_name : String?
    column vendor_vat_number : String?
    column vendor_address : String?

    # Customer info
    column customer_name : String?
    column customer_vat_number : String?
    column customer_address : String?

    # Invoice details
    column invoice_number : String?
    column invoice_date : Time?
    column due_date : Time?

    # Amounts
    column subtotal : Float64?
    column total_tax : Float64?
    column invoice_total : Float64?

    # VIES validation
    column vies_status : String = "pending"  # pending, valid, invalid, not_applicable, error
    column vies_validation_message : String?
    column vies_company_name : String?
    column vies_company_address : String?
    column vies_validated_at : Time?

    # Selected accounts for journal entry
    column counterparty_account_id : Int64?
    column vat_account_id : Int64?
    column expense_revenue_account_id : Int64?

    # Review flags
    column requires_manual_review : Bool = false
    column manual_review_reason : String?
    column notes : String?

    # Processing metadata
    column confidence : Float64?
    column original_file_name : String?
    column s3_key : String?
    column s3_key_json : String?
    column azure_raw_json : String?
    column vat_period : String?         # YYYY-MM format
    column has_inventory : Bool = false

    belongs_to company : Company
    belongs_to journal_entry : JournalEntry?
    belongs_to counterpart : Counterpart?
  end
end
