class WarehouseDocumentLine < BaseModel
  table do
    column quantity : Float64
    column unit_code : String = "C62"
    column unit_price : Float64 = 0.0
    column line_total : Float64 = 0.0
    column line_number : Int32 = 1

    belongs_to warehouse_document : WarehouseDocument
    belongs_to product : Product
  end
end
