class Api::VatRates::Index < ApiAction
  get "/api/vat_rates" do
    vat_rates = VatRateQuery.new.is_active(true)

    json({
      success: true,
      data:    vat_rates.map { |v| vat_rate_json(v) },
    })
  end

  private def vat_rate_json(v : VatRate)
    {
      id:             v.id,
      name:           v.name,
      percentage:     v.percentage,
      code:           v.code,
      saft_tax_type:  v.saft_tax_type,
      saft_tax_code:  v.saft_tax_code,
      effective_from: v.effective_from,
      effective_to:   v.effective_to,
      is_active:      v.is_active,
    }
  end
end
