class Api::CompanyLocations::Create < ApiAction
  post "/api/companies/:company_id/locations" do
    SaveCompanyLocation.create(params, company_id: company_id.to_i64) do |operation, loc|
      if loc
        json({
          success: true,
          data:    {
            id:              loc.id,
            name:            loc.name,
            location_type:   loc.location_type,
            street_name:     loc.street_name,
            building_number: loc.building_number,
            city:            loc.city,
            post_code:       loc.post_code,
            region:          loc.region,
            country:         loc.country,
            phone:           loc.phone,
            email:           loc.email,
            is_main:         loc.is_main,
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
