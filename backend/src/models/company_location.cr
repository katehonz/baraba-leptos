# Търговски обекти и локации на компанията
# Използва се за SAFt адресна структура
class CompanyLocation < BaseModel
  table do
    column name : String                    # Име на обекта
    column location_type : String           # OFFICE, STORE, WAREHOUSE, BRANCH
    column street_name : String?            # Улица
    column building_number : String?        # Номер
    column city : String?                   # Град
    column post_code : String?              # Пощенски код
    column region : String?                 # ISO 3166-2:BG
    column country : String = "BG"          # Държава
    column phone : String?                  # Телефон
    column email : String?                  # Email
    column is_main : Bool = false           # Главен адрес (седалище)
    column is_active : Bool = true

    belongs_to company : Company
  end

  LOCATION_TYPES = {
    "OFFICE"    => "Офис",
    "STORE"     => "Магазин",
    "WAREHOUSE" => "Склад",
    "BRANCH"    => "Клон/Поделение",
  }

  def type_name : String
    LOCATION_TYPES[location_type]? || location_type
  end

  def full_address : String
    parts = [] of String
    parts << street_name.not_nil! if street_name
    parts << "№#{building_number}" if building_number
    parts << city.not_nil! if city
    parts << post_code.not_nil! if post_code
    parts.join(", ")
  end
end
