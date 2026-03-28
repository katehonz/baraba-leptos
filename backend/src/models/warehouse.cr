class Warehouse < BaseModel
  table do
    column name : String
    column address : String?
    column description : String?
    column is_active : Bool = true

    belongs_to company : Company
    has_many stock_transactions : StockTransaction
  end
end
