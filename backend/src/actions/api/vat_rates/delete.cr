class Api::VatRates::Delete < ApiAction
  delete "/api/vat_rates/:vat_rate_id" do
    vat_rate = VatRateQuery.new.find(vat_rate_id)
    DeleteVatRate.delete!(vat_rate)

    json({
      success: true,
      message: "VAT rate deleted successfully",
    })
  end
end
