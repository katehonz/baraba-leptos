# SAF-T Cash Movement Type reference table
# Номенклатура за движение на парични средства (каси и банки)
class SaftCashMovementType < BaseModel
  table do
    column code : String
    column name_bg : String
    column name_en : String?
    column payment_method_code : String?
  end

  def display_name : String
    "#{code} - #{name_bg}"
  end
end
