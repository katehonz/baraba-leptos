class Api::CompanyLocations::Update < ApiAction
  put "/api/companies/:company_id/locations/:id" do
    loc = CompanyLocationQuery.new.id(id).company_id(company_id).first?

    if loc
      SaveCompanyLocation.update(loc, params) do |operation, updated|
        if updated
          json({
            success: true,
            data:    {
              id:              updated.id,
              name:            updated.name,
              location_type:   updated.location_type,
              street_name:     updated.street_name,
              building_number: updated.building_number,
              city:            updated.city,
              post_code:       updated.post_code,
              region:          updated.region,
              country:         updated.country,
              phone:           updated.phone,
              email:           updated.email,
              is_main:         updated.is_main,
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
    else
      response.status_code = 404
      json({success: false, message: "Not found"})
    end
  end
end
