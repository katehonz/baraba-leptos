class WarehouseScan < BaseModel
  table do
    column direction : String = "inbound" # inbound, outbound
    column status : String = "pending"    # pending, reviewed, applied, rejected
    column original_file_name : String?
    column s3_key : String?
    column raw_json : String = "{}"
    column vendor_name : String?
    column vendor_vat : String?
    column invoice_number : String?
    column document_date : Time?
    column subtotal : Float64?
    column vat_amount : Float64?
    column total_amount : Float64?
    column confidence : Float64?
    column notes : String?

    belongs_to company : Company
    belongs_to counterpart : Counterpart?
    belongs_to warehouse_document : WarehouseDocument?
    belongs_to invoice : Invoice?
  end

  DIRECTIONS = ["inbound", "outbound"]
  STATUSES   = ["pending", "reviewed", "applied", "rejected"]
end
