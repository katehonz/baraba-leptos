class Api::Currencies::Show < ApiAction
  get "/api/currencies/:currency_id" do
    currency = CurrencyQuery.new.find(currency_id)

    json({
      success: true,
      data:    {
        id:               currency.id,
        code:             currency.code,
        name:             currency.name,
        name_bg:          currency.name_bg,
        symbol:           currency.symbol,
        decimal_places:   currency.decimal_places,
        is_active:        currency.is_active,
        is_base_currency: currency.is_base_currency,
      },
    })
  end
end
