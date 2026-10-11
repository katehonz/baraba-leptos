# Resolves the multiplier stored on Invoice.exchange_rate:
#   functional_amount = Money.round(document_amount * exchange_rate)
# ECB rows are stored as 1 EUR = rate units of the quoted currency.
module ExchangeRateLookup
  extend self

  def for_currency(code : String, on_date : Time, company_currency : String)
    doc = code.upcase.strip
    func = company_currency.upcase.strip
    date_str = on_date.to_s("%Y-%m-%d")

    if doc == func
      return result(doc, func, date_str, 1.0, 1.0, "identity", true)
    end

    if bgn_eur?(doc, func)
      multiplier = if doc == "BGN" && func == "EUR"
                     1.0 / Money.bgn_eur_rate
                   else
                     Money.bgn_eur_rate
                   end
      return result(doc, func, date_str, Money.bgn_eur_rate, multiplier, "fixed", true)
    end

    eur_per_doc, doc_date, doc_ecb = eur_per_unit(doc, on_date)
    eur_per_func, _func_date, _func_ecb = eur_per_unit(func, on_date)

    unless eur_per_doc && eur_per_func && eur_per_func != 0.0
      return result(doc, func, date_str, 0.0, 1.0, "missing", false)
    end

    multiplier = eur_per_doc / eur_per_func
    rate_date = doc_date.try(&.to_s("%Y-%m-%d")) || date_str
    ecb_display = doc_ecb || (1.0 / eur_per_doc)
    result(doc, func, rate_date, ecb_display, multiplier, "ecb", true)
  end

  private def bgn_eur?(a : String, b : String) : Bool
    (a == "BGN" && b == "EUR") || (a == "EUR" && b == "BGN")
  end

  # How many EUR is 1 unit of `code`, plus the ECB quote (1 EUR = X code) when applicable.
  private def eur_per_unit(code : String, on_date : Time) : {Float64?, Time?, Float64?}
    return {1.0, on_date, 1.0} if code == "EUR"
    if code == "BGN"
      return {1.0 / Money.bgn_eur_rate, on_date, Money.bgn_eur_rate}
    end

    quoted = latest_ecb_quote(code, on_date)
    return {nil, nil, nil} unless quoted
    rate, at = quoted
    return {nil, nil, nil} if rate == 0.0
    {1.0 / rate, at, rate}
  end

  private def latest_ecb_quote(to_code : String, on_date : Time) : {Float64, Time}?
    eur = CurrencyQuery.new.code("EUR").first?
    target = CurrencyQuery.new.code(to_code).first?
    return nil unless eur && target

    end_of_day = Time.utc(on_date.year, on_date.month, on_date.day, 23, 59, 59)
    row = ExchangeRateQuery.new
      .from_currency_id(eur.id)
      .to_currency_id(target.id)
      .is_active(true)
      .valid_date.lte(end_of_day)
      .valid_date.desc_order
      .first?
    return nil unless row
    {row.rate, row.valid_date}
  end

  private def result(code, company_currency, rate_date, ecb_rate, exchange_rate, source, found)
    {
      code:              code,
      company_currency:  company_currency,
      rate_date:         rate_date,
      ecb_rate:          ecb_rate,
      exchange_rate:     exchange_rate,
      source:            source,
      found:             found,
    }
  end
end
