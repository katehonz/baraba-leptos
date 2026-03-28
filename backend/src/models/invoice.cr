class Invoice < BaseModel
  skip_schema_enforcer
  table do
    column invoice_number : String
    column document_type : String = "INVOICE"  # INVOICE, DEBIT_NOTE, CREDIT_NOTE
    column original_invoice_id : Int64?

    column issue_date : Time
    column due_date : Time?

    column subtotal : Float64
    column vat_amount : Float64
    column total_amount : Float64

    column currency : String = "BGN"
    column exchange_rate : Float64 = 1.0
    column payment_method : String?  # BANK, CASH
    column notes : String?
    column status : String = "draft"  # draft, posted, cancelled

    # VAT Compliance Fields
    column tax_event_date : Time?
    column vat_exemption_reason : String?
    column has_inventory : Bool = false

    # SAF-T Invoice fields
    column saft_invoice_type : String?       # Code from Nom_Invoice_Types (1, 2, 3, 7, 9, etc.)
    column period : Int32?                   # Accounting period (1-12)
    column period_year : Int32?              # Year of accounting period
    column transaction_id : String?          # Unique transaction ID (required for SAF-T)
    column branch_store_number : String?     # Branch/store number
    column self_billing_indicator : Bool = false
    column gl_posting_date : Time?           # GL posting date
    column batch_id : String?                # Batch ID
    column system_id : String?               # System ID
    column source_id : String?               # Source (user/module that created)

    # SAF-T CustomerID/SupplierID (formatted)
    column saft_customer_id : String?
    column saft_supplier_id : String?

    # Lines stored as JSON string (parse at runtime for compatibility)
    column lines : String = "[]"

    belongs_to company : Company
    belongs_to counterpart : Counterpart
    belongs_to journal_entry : JournalEntry?
    belongs_to bank_account : BankAccount?

    # Keep has_many for backward compatibility
    has_many invoice_lines : InvoiceLine
    has_many payment_lines : PaymentLine
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
