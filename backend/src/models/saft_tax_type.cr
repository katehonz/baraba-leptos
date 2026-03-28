# SAF-T Tax Type reference table
# Used for Bulgarian tax reporting (SAF-T BG)
class SaftTaxType < BaseModel
  table do
    column code : String
    column name : String
    column description : String?
  end
end
