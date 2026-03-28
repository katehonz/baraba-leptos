class Api::Companies::Index < ApiAction
  include Api::Auth::RequireAuthToken

  get "/api/companies" do
    # Authorization: require super_admin for listing all companies
    user = current_user?.not_nil!
    unless user.is_super_admin
      response.status_code = 403
      return json({success: false, message: "Само супер администратори имат достъп до списъка с всички фирми"})
    end
    # Get filter parameters
    query_param = params.get?(:q)
    city_filter = params.get?(:city)
    page = params.get?(:page).try(&.to_i) || 1
    per_page = params.get?(:per_page).try(&.to_i) || 50

    # Build query
    companies_query = CompanyQuery.new

    # Text search: name, eik, vat_number
    if query_param && !query_param.empty?
      search_term = "%#{query_param.downcase}%"
      companies_query = companies_query.where_ilike_name(search_term)
        .or(&.where_ilike_eik(search_term))
        .or(&.where_ilike_vat_number(search_term))
    end

    # City filter
    if city_filter && !city_filter.empty?
      companies_query = companies_query.city(city_filter)
    end

    # Get total count before pagination
    total = companies_query.clone.select_count

    # Apply pagination
    offset = (page - 1) * per_page
    companies_query = companies_query.limit(per_page).offset(offset)

    # Get companies
    companies = companies_query.to_a

    json({
      success: true,
      data:    companies.map { |c| company_json(c) },
      meta:    {
        total:    total,
        page:     page,
        per_page: per_page,
        pages:    (total / per_page.to_f).ceil.to_i,
      },
    })
  end

  private def company_json(company : Company)
    # Count users in this company
    user_count = UserCompanyRoleQuery.new.company_id(company.id).select_count

    {
      id:                     company.id,
      name:                   company.name,
      eik:                    company.eik,
      vat_number:             company.vat_number,
      address:                company.address,
      city:                   company.city,
      postal_code:            company.post_code,
      email:                  company.email,
      phone:                  company.phone,
      manager_name:           company.manager_name,
      manager_egn:            company.manager_egn,
      authorized_person_name: company.authorized_person_name,
      authorized_person_egn:  company.authorized_person_egn,
      representative_type:    company.representative_type,
      vat_branch_number:      company.vat_branch_number,
      user_count:             user_count,
      created_at:             company.created_at,
    }
  end
end

# Extend CompanyQuery with search methods
class CompanyQuery
  def where_ilike_name(pattern : String)
    name.ilike(pattern)
  end

  def where_ilike_eik(pattern : String)
    eik.ilike(pattern)
  end

  def where_ilike_vat_number(pattern : String)
    vat_number.ilike(pattern)
  end
end
