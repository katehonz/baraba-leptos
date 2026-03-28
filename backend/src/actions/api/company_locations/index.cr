class Api::CompanyLocations::Index < ApiAction
  get "/api/companies/:company_id/locations" do
    locations = CompanyLocationQuery.new
      .company_id(company_id)
      .is_active(true)

    json({
      success: true,
      data:    locations.map { |loc| serialize(loc) },
    })
  end

  private def serialize(loc : CompanyLocation)
    {
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
    }
  end
end
