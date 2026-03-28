# SAF-T Standard Account from NRA nomenclature.
# This is the official chart of accounts for SAF-T reporting in Bulgaria.
class SaftAccount < BaseModel
  table do
    column code : String                    # e.g., "101", "4531"
    column name : String                    # e.g., "Основен капитал, изискващ регистрация"
    column section_code : String?           # e.g., "1" for section 1
    column section_name : String?           # e.g., "Сметки за капитал и заеми"
    column group_code : String?             # e.g., "10" for group 10
    column group_name : String?             # e.g., "Капитал"
  end

  # Display name for dropdowns: "101 - Основен капитал, изискващ регистрация"
  def display_name : String
    "#{code} - #{name}"
  end
end
