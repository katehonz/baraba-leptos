# SAF-T Stock Movement Type reference table
# Номенклатура за движение на продукти (материални запаси)
class SaftStockMovementType < BaseModel
  table do
    column code : String
    column name_bg : String
    column name_en : String?
  end

  def display_name : String
    "#{code} - #{name_bg}"
  end
end
