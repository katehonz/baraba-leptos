class NapDocumentType < BaseModel
  table do
    column code : String
    column journal : String
    column name_bg : String
    column saft_invoice_type : String?
    column is_tax : Bool = true
  end
end
