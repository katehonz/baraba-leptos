class Api::Currencies::Index < ApiAction
  get "/api/currencies" do
    currencies = CurrencyQuery.new.is_active(true)

    json({
      success: true,
      data:    currencies.map { |c| currency_json(c) },
    })
  end

  private def currency_json(c : Currency)
    {
      id:               c.id,
      code:             c.code,
      name:             c.name,
      name_bg:          c.name_bg,
      symbol:           c.symbol,
      decimal_places:   c.decimal_places,
      is_active:        c.is_active,
      is_base_currency: c.is_base_currency,
    }
  end
end
