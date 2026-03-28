class Api::Nomenclatures::Iso::Currencies < ApiAction
  get "/api/nomenclatures/iso/currencies" do
    currencies = IsoCurrencyQuery.new

    json({
      success: true,
      data:    currencies.map { |c| currency_json(c) },
    })
  end

  private def currency_json(c : IsoCurrency)
    {
      id:             c.id,
      code:           c.code,
      name:           c.name,
      name_bg:        c.name_bg,
      symbol:         c.symbol,
      decimal_places: c.decimal_places,
    }
  end
end
