class Api::ExchangeRates::Show < ApiAction
  get "/api/exchange_rates/:exchange_rate_id" do
    exchange_rate = ExchangeRateQuery.new.find(exchange_rate_id)

    json({
      success: true,
      data:    {
        id:               exchange_rate.id,
        from_currency_id: exchange_rate.from_currency_id,
        to_currency_id:   exchange_rate.to_currency_id,
        rate:             exchange_rate.rate,
        reverse_rate:     exchange_rate.reverse_rate,
        valid_date:       exchange_rate.valid_date,
        rate_source:      exchange_rate.rate_source,
        is_active:        exchange_rate.is_active,
        notes:            exchange_rate.notes,
      },
    })
  end
end
