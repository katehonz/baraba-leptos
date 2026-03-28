require "../../../services/vies_service"

class Api::Vies::Check < ApiAction
  # POST /api/vies/check
  # Body: { "vat_number": "BG000000000" }
  post "/api/vies/check" do
    vat_number = params.from_json["vat_number"]?.try(&.as_s) || ""

    if vat_number.empty?
      response.status_code = 400
      return json({success: false, error: "VAT number is required"})
    end

    result = ViesService.check_vat(vat_number)

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
