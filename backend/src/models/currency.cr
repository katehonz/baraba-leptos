class Currency < BaseModel
  table do
    column code : String              # ISO 4217 code (EUR, USD, BGN)
    column name : String              # English name
    column name_bg : String?          # Bulgarian name
    column symbol : String?           # Currency symbol
    column decimal_places : Int32 = 2
    column is_active : Bool = true
    column is_base_currency : Bool = false
  end
end
