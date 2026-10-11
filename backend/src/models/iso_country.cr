# ISO 3166-1 Country codes
class IsoCountry < BaseModel
  table do
    column alpha2 : String      # BG, DE, FR
    column alpha3 : String      # BGR, DEU, FRA
    column name : String        # Bulgaria, Germany, France
    column name_bg : String?    # България, Германия, Франция
    column numeric : String?    # 100, 276, 250
    column is_eu_member : Bool = false
  end
end
