# SAF-T Unit of Measure reference table
# Номенклатура на мерните единици (UN/ECE Recommendation 20)
class SaftUnitOfMeasure < BaseModel
  table :saft_units_of_measure do
    column code : String
    column name_en : String
    column name_bg : String
    column symbol : String?
    column category : String?
    column is_common : Bool = false
    column sort_order : Int32 = 999
  end

  def display_name : String
    "#{code} - #{name_bg}"
  end
end
