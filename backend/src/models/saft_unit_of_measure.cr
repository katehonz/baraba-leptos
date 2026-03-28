# SAF-T Unit of Measure reference table
# Номенклатура на мерните единици (UN/ECE Recommendation 20)
class SaftUnitOfMeasure < BaseModel
  table :saft_units_of_measure do
    column code : String
    column name_en : String
    column name_bg : String
  end

  def display_name : String
    "#{code} - #{name_bg}"
  end
end
