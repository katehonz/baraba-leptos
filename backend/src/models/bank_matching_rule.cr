class BankMatchingRule < BaseModel
  table do
    column name : String
    column keywords : String
    column direction : String = "any"
    column priority : Int32 = 0
    column is_active : Bool = true

    belongs_to company : Company
    belongs_to account : Account
    belongs_to counterpart : Counterpart?
  end
end
