# SAF-T Payment Method reference table
# Used for Bulgarian tax reporting (SAF-T BG)
class SaftPaymentMethod < BaseModel
  table do
    column method_code : String
    column mechanism_code : String
    column name : String
    column description : String?
  end
end
