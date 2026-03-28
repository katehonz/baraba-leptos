# SAF-T PhysicalStock model for MasterFiles > PhysicalStock section (on-demand reporting)
class PhysicalStock < BaseModel
  table do
    column location_id : String?
    column stock_account_no : String?
    column product_type : String          # 10, 20, 30, 40, 50
    column product_status : String?
    column stock_commodity_code : String?
    column owner_id : String              # SAF-T formatted owner ID
    column uom_physical_stock : String
    column uom_conversion_factor : Float64 = 1.0
    column unit_price_begin : Float64 = 0.0
    column unit_price_end : Float64 = 0.0
    column opening_stock_quantity : Float64 = 0.0
    column opening_stock_value : Float64 = 0.0
    column closing_stock_quantity : Float64 = 0.0
    column closing_stock_value : Float64 = 0.0
    column period_start : Time?
    column period_end : Time?
    column characteristics : String?      # JSON array

    belongs_to company : Company
    belongs_to warehouse : Warehouse
    belongs_to product : Product
  end

  # Parse characteristics JSON
  def parsed_characteristics : Array(Hash(String, String))
    return [] of Hash(String, String) if self.characteristics.nil? || self.characteristics.try(&.empty?)
    begin
      JSON.parse(self.characteristics.not_nil!).as_a.map do |item|
        h = item.as_h
        {"code" => h["code"]?.try(&.as_s) || "", "value" => h["value"]?.try(&.as_s) || ""}
      end
    rescue
      [] of Hash(String, String)
    end
  end
end
