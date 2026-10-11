class Invoice < BaseModel
  table do
    column invoice_number : String?
    column document_type : String = "INVOICE" # INVOICE, DEBIT_NOTE, CREDIT_NOTE, PROFORMA
    column direction : String = "sale"        # sale, purchase

    column issue_date : Time
    column due_date : Time?

    column subtotal : Float64
    column vat_amount : Float64
    column total_amount : Float64

    column currency : String = "BGN"
    column exchange_rate : Float64 = 1.0
    column payment_method : String? # BANK, CASH
    column notes : String?
    column status : String = "draft" # draft, issued, posted, cancelled
    column nap_document_type : String?
    column uncl1001_code : String?
    column issued_at : Time?
    column cancelled_at : Time?
    column cancellation_reason : String?

    # VAT Compliance Fields
    column tax_event_date : Time?
    column vat_exemption_reason : String?
    column has_inventory : Bool = false

    # SAF-T Invoice fields
    column saft_invoice_type : String? # Code from Nom_Invoice_Types (1, 2, 3, 7, 9, etc.)
    column period : Int32?             # Accounting period (1-12)
    column period_year : Int32?        # Year of accounting period
    column transaction_id : String?    # Unique transaction ID (required for SAF-T)
    column branch_store_number : String?
    column self_billing_indicator : Bool = false
    column gl_posting_date : Time?
    column batch_id : String?
    column system_id : String?
    column source_id : String?

    column saft_customer_id : String?
    column saft_supplier_id : String?

    # Lines stored as JSON string (parse at runtime for compatibility)
    column lines : String = "[]"

    belongs_to company : Company
    belongs_to counterpart : Counterpart
    belongs_to original_invoice : Invoice?
    belongs_to journal_entry : JournalEntry?
    belongs_to bank_account : BankAccount?
    belongs_to location : CompanyLocation?
    belongs_to sequence : DocumentNumberSequence?

    has_many invoice_lines : InvoiceLine
    has_many payment_lines : PaymentLine
  end

  DIRECTIONS      = ["sale", "purchase"]
  DOCUMENT_TYPES  = ["INVOICE", "DEBIT_NOTE", "CREDIT_NOTE", "PROFORMA", "PROTOCOL", "SALES_REPORT"]
  STATUSES        = ["draft", "issued", "posted", "cancelled"]
  # Проформата не е данъчен документ и не се осчетоводява.
  TAX_DOC_TYPES   = ["INVOICE", "DEBIT_NOTE", "CREDIT_NOTE", "PROTOCOL", "SALES_REPORT"]

  # Вътрешен вид за номерация и осчетоводяване от кода по ППЗДДС.
  def self.kind_for_nap(code : String) : String
    case code
    when "02", "12"                                         then "DEBIT_NOTE"
    when "03", "13", "23"                                   then "CREDIT_NOTE"
    when "81", "82", "83", "84", "85"                       then "SALES_REPORT"
    when "04", "05", "07", "09", "29", "50", "91", "92", "93", "94", "95"
      "PROTOCOL"
    else
      "INVOICE"
    end
  end

  def draft?
    status == "draft"
  end

  def issued?
    status == "issued"
  end

  def posted?
    status == "posted"
  end

  def sale?
    direction == "sale"
  end

  def purchase?
    direction == "purchase"
  end

  # Generate SAF-T transaction ID
  def generate_transaction_id!
    self.transaction_id = "INV-#{self.company_id}-#{Time.utc.to_unix}-#{Random.rand(9999).to_s.rjust(4, '0')}"
  end

  # Set period from issue_date
  def set_period_from_date!
    date = self.issue_date
    self.period = date.month
    self.period_year = date.year
  end

  # Generate SAF-T IDs based on counterpart
  def generate_saft_ids!(counterpart : Counterpart)
    if counterpart.counterpart_type == "customer" || counterpart.counterpart_type == "both"
      self.saft_customer_id = counterpart.saft_id
      self.saft_supplier_id = "0"
    elsif counterpart.counterpart_type == "supplier"
      self.saft_supplier_id = counterpart.saft_id
      self.saft_customer_id = "0"
    end
  end

  # Helper to parse lines from JSON string
  def parsed_lines : JSON::Any
    lines_json = self.lines
    return JSON::Any.new([] of JSON::Any) if lines_json.nil? || lines_json.empty?

    begin
      JSON.parse(lines_json)
    rescue ex : JSON::ParseException
      JSON::Any.new([] of JSON::Any)
    end
  end
end
