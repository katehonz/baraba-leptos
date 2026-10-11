class Api::CompanyLocations::Delete < ApiAction
  delete "/api/companies/:company_id/locations/:id" do
    loc = CompanyLocationQuery.new.id(id).company_id(company_id).first?

    if loc
      SaveCompanyLocation.update!(loc, is_active: false)
      json({success: true})
    else
      response.status_code = 404
      json({success: false, message: "Not found"})
    end
  end
end
