# ISO 4217 Currency codes
class IsoCurrency < BaseModel
  table do
    column code : String        # BGN, EUR, USD
    column name : String        # Bulgarian Lev, Euro, US Dollar
    column name_bg : String?    # Български лев, Евро, Щатски долар
    column numeric : String?    # 975, 978, 840
    column symbol : String?     # лв, €, $
    column decimal_places : Int32 = 2
  end
end
