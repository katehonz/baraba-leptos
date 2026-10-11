class InvoiceLine < BaseModel
  table do
    column description : String
    column quantity : Float64
    column unit_price : Float64
    column vat_rate : Float64 = 20.0
    column line_total : Float64
    column unit_code : String = "C62"
    column vat_amount : Float64 = 0.0
    column line_total_with_vat : Float64 = 0.0
    column line_number : Int32 = 1
    column tax_type : String?
    column tax_code : String?
    column discount_percent : Float64 = 0.0
    column discount_amount : Float64 = 0.0

    belongs_to invoice : Invoice
    belongs_to account : Account?
    belongs_to product : Product?
    belongs_to vat_exemption_reason : VatExemptionReason?
  end
end
