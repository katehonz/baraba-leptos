# Крайно предприятие майка (SAFt S.O.9-S.O.11)
class UltimateParent < BaseModel
  table do
    column name_bg : String                 # Наименование на кирилица
    column name_latin : String?             # Name in Latin
    column uic : String?                    # ЕИК/UIC
    column country : String = "BG"          # ISO country code
    column is_active : Bool = true

    belongs_to company : Company
  end
end
