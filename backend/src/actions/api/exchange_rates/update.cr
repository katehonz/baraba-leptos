class Api::ExchangeRates::Update < ApiAction
  put "/api/exchange_rates/:exchange_rate_id" do
    exchange_rate = ExchangeRateQuery.new.find(exchange_rate_id)

    SaveExchangeRate.update(exchange_rate, params) do |operation, updated|
      if operation.valid?
        json({
          success: true,
          data:    {
            id:               updated.id,
            from_currency_id: updated.from_currency_id,
            to_currency_id:   updated.to_currency_id,
            rate:             updated.rate,
            reverse_rate:     updated.reverse_rate,
            valid_date:       updated.valid_date,
            rate_source:      updated.rate_source,
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
