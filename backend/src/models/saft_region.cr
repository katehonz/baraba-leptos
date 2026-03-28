# SAF-T Region reference table
# ISO 3166-2:BG Bulgarian Area Codes (Области)
class SaftRegion < BaseModel
  table do
    column code : String        # e.g., "BG-01"
    column name : String        # Region name in Bulgarian (e.g., "Благоевград")
  end

  def display_name : String
    "#{code} - #{name}"
  end
end
