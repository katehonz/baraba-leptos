class Api::Users::MyCompanies < ApiAction
  include Api::Auth::RequireAuthToken

  get "/api/my_companies" do
    user = current_user?.not_nil!

    # Super admin sees all companies
    if user.is_super_admin
      companies = CompanyQuery.new.to_a
    else
      # Regular users see only companies they have access to
      user_company_roles = UserCompanyRoleQuery.new.user_id(user.id).preload_company
      companies = user_company_roles.map(&.company)
    end

    json({
      success: true,
      data:    companies.map { |c| company_json(c) },
    })
  end

  private def company_json(company : Company)
    {
      id:   company.id,
      name: company.name,
      eik:  company.eik,
    }
  end
end
