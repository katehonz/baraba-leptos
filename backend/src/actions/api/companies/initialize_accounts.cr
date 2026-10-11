# POST /api/companies/:company_id/initialize_accounts - Initialize accounts from SAF-T
class Api::Companies::InitializeAccounts < ApiAction
  post "/api/companies/:company_id/initialize_accounts" do
    company = CompanyQuery.new.find(company_id)

    result = CompanyAccountsInitializer.initialize_accounts(company)

    json({
      success: result[:success],
      message: result[:message],
      created: result[:created],
    })
  rescue Avram::RecordNotFoundError
    response.status_code = 404
    json({success: false, message: "Фирмата не е намерена"})
  end
end
