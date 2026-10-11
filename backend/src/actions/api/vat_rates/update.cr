class Api::VatRates::Update < ApiAction
  put "/api/vat_rates/:vat_rate_id" do
    vat_rate = VatRateQuery.new.find(vat_rate_id)

    SaveVatRate.update(vat_rate, params) do |operation, updated|
      if operation.valid?
        json({
          success: true,
          data:    {
            id:         updated.id,
            name:       updated.name,
            percentage: updated.percentage,
            code:       updated.code,
            is_active:  updated.is_active,
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
