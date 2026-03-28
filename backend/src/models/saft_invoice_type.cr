# SAF-T Invoice Type reference table
# Used for Bulgarian tax reporting (SAF-T BG)
class SaftInvoiceType < BaseModel
  table do
    column code : String
    column name : String
    column description : String?
  end
end
