# Shareholder - съдружник/акционер
class Shareholder < BaseModel
  table do
    column shareholder_type : String  # person, company
    column name : String
    column identifier : String?       # EGN, LNCH, EIK
    column address : String?
    column percentage_owned : Float64?

    belongs_to company : Company
    has_many dividends : Dividend
  end
end
