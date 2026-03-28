class Api::VatRates::Create < ApiAction
  post "/api/vat_rates" do
    SaveVatRate.create(params) do |operation, vat_rate|
      if vat_rate
        json({
          success: true,
          data:    {
            id:         vat_rate.id,
            name:       vat_rate.name,
            percentage: vat_rate.percentage,
            code:       vat_rate.code,
            is_active:  vat_rate.is_active,
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
