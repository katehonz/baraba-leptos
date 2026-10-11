class Api::VatRates::Show < ApiAction
  get "/api/vat_rates/:vat_rate_id" do
    vat_rate = VatRateQuery.new.find(vat_rate_id)

    json({
      success: true,
      data:    {
        id:             vat_rate.id,
        name:           vat_rate.name,
        percentage:     vat_rate.percentage,
        code:           vat_rate.code,
        saft_tax_type:  vat_rate.saft_tax_type,
        saft_tax_code:  vat_rate.saft_tax_code,
        effective_from: vat_rate.effective_from,
        effective_to:   vat_rate.effective_to,
        is_active:      vat_rate.is_active,
      },
    })
  end
end
