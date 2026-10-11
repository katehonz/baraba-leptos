class WarehouseDocumentType < BaseModel
  table do
    column code : String
    column name_bg : String
    column saft_movement_type : String
    column needs_from : Bool = false
    column needs_to : Bool = false
    column default_is_incoming : Bool?
    column system_only : Bool = false
    column sort_order : Int32 = 100
  end
end
