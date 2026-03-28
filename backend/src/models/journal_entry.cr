class JournalEntry < BaseModel
  skip_schema_enforcer
  table do
    column entry_date : Time
    column description : String
    column reference : String?
    column status : String = "draft"  # draft, posted, reversed

    # VAT Fields
    column document_number : String?
    column document_date : Time?
    column vat_purchase_operation : String?
    column vat_sales_operation : String?
    column total_amount : Float64?
    column total_vat_amount : Float64?
    column payment_method_code : String?
    column vat_period : String?  # Format: YYYY-MM
    column counterpart_id : Int64?
    column branch_code : String = "0001"  # Branch code for multi-branch companies (4 digits)

    # SAF-T Transaction fields
    column transaction_id : String?          # Unique transaction ID (required for SAF-T)
    column period : Int32?                   # Accounting period (1-12)
    column period_year : Int32?              # Year of accounting period
    column journal_id : String?              # Journal identifier (e.g., PAYMENTS, SALES)
    column journal_type : String?            # Journal type (e.g., AP, AR)
    column transaction_type : String?        # Transaction type (normal, periodic, etc.)
    column batch_id : String?                # Batch ID
    column system_entry_date : Time?         # System entry date
    column gl_posting_date : Time?           # GL posting date
    column source_id : String?               # Source (user/module that created)

    # SAF-T CustomerID/SupplierID (formatted)
    column saft_customer_id : String?
    column saft_supplier_id : String?

    # Lines stored as JSON string (parse at runtime for compatibility)
    column lines : String = "[]"

    belongs_to company : Company
    belongs_to user : User?

    has_many documents : Document
    has_many invoices : Invoice
    has_many payments : Payment
    has_many asset_transactions : AssetTransaction
  end

  # Generate SAF-T transaction ID
  def generate_transaction_id!
    self.transaction_id = "JE-#{self.company_id}-#{Time.utc.to_unix}-#{Random.rand(9999).to_s.rjust(4, '0')}"
  end

  # Set period from entry_date
  def set_period_from_date!
    date = self.entry_date
    self.period = date.month
    self.period_year = date.year
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

  # Check if entry is balanced
  def balanced? : Bool
    current = parsed_lines.as_a?
    return false if current.nil? || current.empty?

    total_debit = 0.0
    total_credit = 0.0

    current.each do |line|
      debit = line["debit"]?.try(&.as_f?) || 0.0
      credit = line["credit"]?.try(&.as_f?) || 0.0
      total_debit += debit
      total_credit += credit
    end

    total_debit == total_credit
  end
end
