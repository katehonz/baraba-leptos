# SAF-T Product Type reference table
# Номенклатура за вид на материалния запас
class SaftProductType < BaseModel
  table do
    column code : String
    column name : String
    column description : String?
  end

  def display_name : String
    "#{code} - #{name}"
  end
end
