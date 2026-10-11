# SAF-T Asset Movement Type reference table
# Номенклатура за движение на активи
class SaftAssetMovementType < BaseModel
  table do
    column code : String
    column name_bg : String
    column name_en : String?
  end

  def display_name : String
    "#{code} - #{name_bg}"
  end
end
