# Разпределение на дивиденти - протокол за разпределяне на печалба
class DividendDistribution < BaseModel
  table do
    column year : Int32                    # Финансова година
    column total_amount : Float64          # Обща сума за разпределяне
    column decision_date : Time            # Дата на решение (протокол на ОС)
    column decision_number : String?       # Номер на протокол
    column status : String = "draft"       # draft, approved, partially_paid, paid
    column notes : String?

    belongs_to company : Company
    has_many dividends : Dividend
  end

  STATUSES = {
    "draft"        => "Чернова",
    "approved"     => "Одобрено",
    "partially_paid" => "Частично платено",
    "paid"         => "Изплатено",
  }

  # Стандартна данъчна ставка за дивиденти в България
  TAX_RATE = 5.0

  # Изчислява дивиденти за всички действителни собственици
  def calculate_dividends(beneficial_owners : Array(BeneficialOwner)) : Array(NamedTuple(
    beneficial_owner_id: Int64,
    ownership_percentage: Float64,
    gross_amount: Float64,
    tax_rate: Float64,
    tax_amount: Float64,
    net_amount: Float64
  ))
    # Общ процент на собственост
    total_percentage = beneficial_owners.sum { |bo| bo.ownership_percentage || 0.0 }
    return [] of NamedTuple(
      beneficial_owner_id: Int64,
      ownership_percentage: Float64,
      gross_amount: Float64,
      tax_rate: Float64,
      tax_amount: Float64,
      net_amount: Float64
    ) if total_percentage == 0.0

    beneficial_owners.compact_map do |owner|
      percentage = owner.ownership_percentage
      next nil unless percentage && percentage > 0

      # Пропорционален дял от общата сума
      gross_amount = (total_amount * percentage / total_percentage).round(2)
      tax_amount = (gross_amount * TAX_RATE / 100).round(2)
      net_amount = gross_amount - tax_amount

      {
        beneficial_owner_id: owner.id,
        ownership_percentage: percentage,
        gross_amount: gross_amount,
        tax_rate: TAX_RATE,
        tax_amount: tax_amount,
        net_amount: net_amount,
      }
    end
  end
end
