class StockBalance < BaseModel
  table do
    column qty : Float64 = 0.0
    column avg_cost : Float64 = 0.0

    belongs_to company : Company
    belongs_to product : Product
    belongs_to warehouse : Warehouse
  end
end
