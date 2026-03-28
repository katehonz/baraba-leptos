class Api::Companies::Delete < ApiAction
  delete "/api/companies/:company_id" do
    company = CompanyQuery.new.id(company_id).first
    DeleteCompany.delete!(company)

    json({success: true, message: "Company deleted"})
  end
end
