class Api::ExchangeRates::Create < ApiAction
  post "/api/exchange_rates" do
    SaveExchangeRate.create(params) do |operation, exchange_rate|
      if exchange_rate
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
          },
        })
      else
        response.status_code = 422
        json({
          success: false,
          errors:  operation.errors.map { |key, errors| {field: key, messages: errors} },
        })
      end
    end
  end
end
