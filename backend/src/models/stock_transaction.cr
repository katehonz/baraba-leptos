class StockTransaction < BaseModel
  table do
    column transaction_date : Time
    column quantity : Float64
    column unit_price : Float64
    column total_amount : Float64
    column is_incoming : Bool  # true = incoming, false = outgoing
    column movement_type : String?  # SAF-T movement type code from Stock_movement
    column document_number : String?
    column document_date : Time?

    # SAF-T MovementOfGoods fields
    column movement_reference : String?      # Unique movement ID
    column movement_posting_date : Time?
    column movement_posting_time : Time?
    column tax_point_date : Time?
    column movement_subtype : String?        # Subtype description
    column source_id : String?               # User/system that created
    column system_id : String?

    # Document reference
    column document_type : String?
    column document_line : String?

    # SAF-T Line fields
    column line_number : Int32?
    column transaction_id : String?          # Links to GL

    # SAF-T CustomerID/SupplierID
    column saft_customer_id : String?
    column saft_supplier_id : String?

    # Shipping points (JSON for ShippingPointStructure)
    column ship_to : String?
    column ship_from : String?

    # Stock fields
    column stock_account_no : String?        # Batch/serial number
    column unit_of_measure : String?         # SAF-T code
    column uom_conversion_factor : Float64 = 1.0
    column book_value : Float64?
    column movement_comments : String?

    # Tax information
    column tax_type : String?
    column tax_code : String?
    column tax_percentage : Float64?
    column tax_base : Float64?
    column tax_amount : Float64?

    belongs_to product : Product
    belongs_to company : Company
    belongs_to warehouse : Warehouse
    belongs_to journal_entry : JournalEntry?
    belongs_to account : Account?
  end

  # Generate movement reference
  def generate_movement_reference!
    self.movement_reference = "SM-#{self.company_id}-#{Time.utc.to_unix}-#{Random.rand(9999).to_s.rjust(4, '0')}"
  end

  # SAF-T quantity sign: positive for incoming, negative for outgoing
  def saft_quantity : Float64
    self.is_incoming ? self.quantity : -self.quantity
  end
end
