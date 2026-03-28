class Api::Companies::Show < ApiAction
  get "/api/companies/:company_id" do
    company = CompanyQuery.new.id(company_id).first?

    if company
      json({
        success: true,
        data:    JsonbSerializer.serialize_company(company),
      })
    else
      response.status_code = 404
      json({success: false, error: "Company not found"})
    end
  end
end
