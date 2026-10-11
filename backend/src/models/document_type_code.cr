class DocumentTypeCode < BaseModel
  table do
    column code : String
    column name : String
    column name_en : String
    column applies_to : String
    column is_common : Bool = false
    column sort_order : Int32 = 100
  end
end
