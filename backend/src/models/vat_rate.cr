class VatRate < BaseModel
  table do
    column name : String
    column percentage : Float64
    column code : String
    column saft_tax_type : String?
    column saft_tax_code : String?
    column effective_from : Time?
    column effective_to : Time?
    column is_active : Bool = true
  end
end
