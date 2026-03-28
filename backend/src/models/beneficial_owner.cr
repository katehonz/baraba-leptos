# Действителен собственик на компания (SAFt S.O.3-S.O.8)
class BeneficialOwner < BaseModel
  table do
    column first_name_bg : String           # Име на кирилица
    column last_name_bg : String            # Фамилия на кирилица
    column egn : String?                    # ЕГН (за български граждани)
    column first_name_latin : String?       # First name in Latin
    column last_name_latin : String?        # Last name in Latin
    column country : String = "BG"          # ISO country code
    column ownership_percentage : Float64?  # Процент на притежание
    column is_active : Bool = true

    belongs_to company : Company
    has_many dividends : Dividend
  end

  def full_name_bg : String
    "#{first_name_bg} #{last_name_bg}"
  end

  def full_name_latin : String?
    return nil unless first_name_latin && last_name_latin
    "#{first_name_latin} #{last_name_latin}"
  end
end
