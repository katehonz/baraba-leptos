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
  def calculate_dividends(beneficial_owners : Array(BeneficialOwner), tax_rate : Float64 = TAX_RATE) : Array(NamedTuple(
    beneficial_owner_id: Int64,
    ownership_percentage: Float64,
    gross_amount: Float64,
    tax_rate: Float64,
    tax_amount: Float64,
    net_amount: Float64
  ))
    empty = [] of NamedTuple(
      beneficial_owner_id: Int64,
      ownership_percentage: Float64,
      gross_amount: Float64,
      tax_rate: Float64,
      tax_amount: Float64,
      net_amount: Float64
    )

    owners = beneficial_owners.select { |bo| (bo.ownership_percentage || 0.0) > 0 }
    total_percentage = owners.sum { |bo| bo.ownership_percentage || 0.0 }
    return empty if total_percentage == 0.0

    rate = tax_rate < 0 ? 0.0 : tax_rate
    results = owners.map do |owner|
      percentage = owner.ownership_percentage.not_nil!
      gross_amount = (total_amount * percentage / total_percentage).round(2)
      tax_amount = (gross_amount * rate / 100).round(2)

      {
        beneficial_owner_id:    owner.id,
        ownership_percentage:   percentage,
        gross_amount:           gross_amount,
        tax_rate:               rate,
        tax_amount:             tax_amount,
        net_amount:             gross_amount - tax_amount,
      }
    end

    # Остатъкът от закръглянето отива при последния собственик, за да се запази общата сума
    if results.size > 1
      assigned = results[0...-1].sum { |r| r[:gross_amount] }
      last_gross = (total_amount - assigned).round(2)
      last_tax = (last_gross * rate / 100).round(2)
      last = results[-1]
      results[-1] = {
        beneficial_owner_id:  last[:beneficial_owner_id],
        ownership_percentage: last[:ownership_percentage],
        gross_amount:         last_gross,
        tax_rate:             rate,
        tax_amount:           last_tax,
        net_amount:           last_gross - last_tax,
      }
    end

    results
  end
end
