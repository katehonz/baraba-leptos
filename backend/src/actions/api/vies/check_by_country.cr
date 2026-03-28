require "../../../services/vies_service"

class Api::Vies::CheckByCountry < ApiAction
  # GET /api/vies/check/:country_code/:vat_number
  get "/api/vies/check/:country_code/:vat_number" do
    result = ViesService.check_vat(country_code.upcase, vat_number)

    if error = result.error
      json({
        success: false,
        error:   error,
        data:    {
          valid:        false,
          country_code: result.country_code,
          vat_number:   result.vat_number,
        },
      })
    else
      json({
        success: true,
        data:    {
          valid:        result.valid,
          name:         result.name,
          address:      result.address,
          country_code: result.country_code,
          vat_number:   result.vat_number,
        },
      })
    end
  end
end
