# SAF-T Payment model for SourceDocuments > Payments section
class Payment < BaseModel
  table do
    column payment_ref_no : String
    column period : Int32?
    column period_year : Int32?
    column transaction_id : String
    column transaction_date : Time
    column payment_method : String        # 01=Cash, 02=Offset, 03=Cashless
    column description : String
    column batch_id : String?
    column system_id : String?
    column source_id : String?
    column saft_customer_id : String?
    column saft_supplier_id : String?
    column net_total : Float64?
    column gross_total : Float64
    column currency_code : String = "BGN"
    column exchange_rate : Float64 = 1.0

    belongs_to company : Company
    belongs_to counterpart : Counterpart?
    belongs_to journal_entry : JournalEntry?
    belongs_to bank_account : BankAccount?
    has_many payment_lines : PaymentLine
  end

  # Generate SAF-T CustomerID/SupplierID based on counterpart
  def generate_saft_ids(counterpart : Counterpart?)
    return unless counterpart
    if counterpart.counterpart_type == "customer" || counterpart.counterpart_type == "both"
      self.saft_customer_id = counterpart.saft_id
      self.saft_supplier_id = "0"
    elsif counterpart.counterpart_type == "supplier"
      self.saft_supplier_id = counterpart.saft_id
      self.saft_customer_id = "0"
    end
  end
end
