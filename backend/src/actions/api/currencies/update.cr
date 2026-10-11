class Api::Currencies::Update < ApiAction
  put "/api/currencies/:currency_id" do
    currency = CurrencyQuery.new.find(currency_id)

    SaveCurrency.update(currency, params) do |operation, updated_currency|
      if operation.valid?
        json({
          success: true,
          data:    {
            id:               updated_currency.id,
            code:             updated_currency.code,
            name:             updated_currency.name,
            name_bg:          updated_currency.name_bg,
            symbol:           updated_currency.symbol,
            decimal_places:   updated_currency.decimal_places,
            is_active:        updated_currency.is_active,
            is_base_currency: updated_currency.is_base_currency,
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
