class ExchangeRate < BaseModel
  table do
    column rate : Float64
    column reverse_rate : Float64?
    column valid_date : Time
    column rate_source : String = "manual"  # ecb, manual, fixed
    column is_active : Bool = true
    column notes : String?

    belongs_to from_currency : Currency
    belongs_to to_currency : Currency
    belongs_to created_by : User?
  end
end
