# Dividend - дивидент за конкретен собственик
class Dividend < BaseModel
  table do
    column gross_amount : Float64
    column tax_rate : Float64         # Usually 5.00%
    column tax_amount : Float64
    column net_amount : Float64
    column decision_date : Time
    column payment_date : Time?
    column is_paid : Bool = false
    column notes : String?

    belongs_to company : Company
    belongs_to shareholder : Shareholder?
    belongs_to beneficial_owner : BeneficialOwner?
    belongs_to dividend_distribution : DividendDistribution?
  end
end
