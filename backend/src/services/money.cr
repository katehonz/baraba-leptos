module Money
  extend self

  def round(value : Float64) : Float64
    # commercial half-up to 2 decimals (not Crystal banker's Float64#round)
    scaled = value * 100.0
    int_part = scaled.trunc
    frac = scaled - int_part
    if frac.abs > 0.499999999 || (frac.abs - 0.5).abs < 1e-12
      int_part += (scaled >= 0 ? 1.0 : -1.0)
    end
    int_part / 100.0
  end

  def bgn_eur_rate : Float64
    1.95583
  end

  def to_functional(amount : Float64, currency : String, company_currency : String, exchange_rate : Float64) : Float64
    return round(amount) if currency == company_currency
    if (currency == "BGN" && company_currency == "EUR") || (currency == "EUR" && company_currency == "BGN")
      if currency == "BGN"
        return round(amount / bgn_eur_rate)
      else
        return round(amount * bgn_eur_rate)
      end
    end
    round(amount * exchange_rate)
  end

  def saft_tax_code(vat_rate : Float64, ubl_category : String?) : String
    cat = (ubl_category || "").upcase
    return "100215" if cat == "K"
    return "100214" if cat == "G"
    return "100219" if cat == "E"
    return "100211" if vat_rate >= 19.0
    return "100213" if vat_rate >= 8.0
    "100214"
  end
end
