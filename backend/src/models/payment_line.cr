# SAF-T PaymentLine model for SourceDocuments > Payments > Line section
class PaymentLine < BaseModel
  table do
    column line_number : Int32?
    column source_document_id : String?
    column description : String?
    column debit_credit_indicator : String  # D or C
    column tax_point_date : Time?
    column amount : Float64
    column currency_code : String = "BGN"
    column currency_amount : Float64?
    column exchange_rate : Float64 = 1.0
    column saft_customer_id : String?
    column saft_supplier_id : String?
    column tax_type : String?
    column tax_code : String?
    column tax_percentage : Float64?
    column tax_base : Float64?
    column tax_amount : Float64?

    belongs_to payment : Payment
    belongs_to account : Account?
    belongs_to invoice : Invoice?
  end
end
