class WarehouseScanLine < BaseModel
  table do
    column raw_name : String?
    column raw_sku : String?
    column raw_barcode : String?
    column quantity : Float64 = 1.0
    column raw_unit : String?
    column unit_code : String = "C62"
    column unit_price : Float64 = 0.0
    column vat_rate : Float64 = 20.0
    column mapping_status : String = "unmapped" # auto, manual, unmapped
    column confidence : Float64?
    column line_number : Int32 = 1

    belongs_to warehouse_scan : WarehouseScan
    belongs_to product : Product?
  end
end
