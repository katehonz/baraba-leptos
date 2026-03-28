class InvoiceLine < BaseModel
  table do
    column description : String
    column quantity : Float64
    column unit_price : Float64
    column vat_rate : Float64 = 20.0
    column line_total : Float64

    belongs_to invoice : Invoice
    belongs_to account : Account?
  end
end
