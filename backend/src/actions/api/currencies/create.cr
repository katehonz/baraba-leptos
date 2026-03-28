class Api::Currencies::Create < ApiAction
  post "/api/currencies" do
    SaveCurrency.create(params) do |operation, currency|
      if currency
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
