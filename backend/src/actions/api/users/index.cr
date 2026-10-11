class Api::Users::Index < ApiAction
  include Api::Auth::RequireAuthToken

  get "/api/users" do
    # Authorization: require super_admin for listing all users
    user = current_user?.not_nil!
    unless user.is_super_admin
      response.status_code = 403
      return json({success: false, message: "Само супер администратори имат достъп до списъка с всички потребители"})
    end

    # Get filter parameters
    query_param = params.get?(:q)
    role_filter = params.get?(:role)
    company_filter = params.get?(:company_id).try(&.to_i64)
    active_filter = params.get?(:is_active)
    page = params.get?(:page).try(&.to_i) || 1
    per_page = params.get?(:per_page).try(&.to_i) || 50

    # Build query
    users_query = UserQuery.new

    # Text search: email, first_name, last_name
    if query_param && !query_param.empty?
      search_term = "%#{query_param.downcase}%"
      users_query = users_query.where_ilike_email(search_term)
        .or(&.where_ilike_first_name(search_term))
        .or(&.where_ilike_last_name(search_term))
    end

    # Active status filter
    if active_filter
      is_active = active_filter == "true" || active_filter == "1"
      users_query = users_query.is_active(is_active)
    end

    # Get total count before pagination
    total = users_query.clone.select_count

    # Apply pagination
    offset = (page - 1) * per_page
    users_query = users_query.limit(per_page).offset(offset)

    # Convert to array and filter by role/company if needed
    users = users_query.to_a

    # Filter by role (requires joining with user_company_roles)
    if role_filter && !role_filter.empty?
      users = users.select do |user|
        user_roles = UserCompanyRoleQuery.new.user_id(user.id).preload_role
        user_roles.any? { |ucr| ucr.role.name == role_filter }
      end
    end

    # Filter by company
    if company_filter
      users = users.select do |user|
        user_roles = UserCompanyRoleQuery.new.user_id(user.id)
        user_roles.any? { |ucr| ucr.company_id == company_filter }
      end
    end

    json({
      success:  true,
      data:     users.map { |u| user_json(u) },
      meta:     {
        total:    total,
        page:     page,
        per_page: per_page,
        pages:    (total / per_page.to_f).ceil.to_i,
      },
    })
  end

  private def user_json(user : User)
    # Get all company roles for this user
    user_roles = UserCompanyRoleQuery.new.user_id(user.id).preload_role.preload_company
    company_roles = user_roles.map do |ucr|
      {
        company_id:   ucr.company_id,
        company_name: ucr.company.name,
        role:         ucr.role.name,
      }
    end

    # Get primary role (first one or "user")
    primary_role = user_roles.first?.try(&.role.try(&.name)) || "user"

    {
      id:             user.id,
      email:          user.email,
      first_name:     user.first_name,
      last_name:      user.last_name,
      role:           primary_role,
      is_active:      user.is_active,
      is_super_admin: user.is_super_admin,
      company_roles:  company_roles,
      created_at:     user.created_at,
    }
  end
end

# Extend UserQuery with search methods
class UserQuery
  def where_ilike_email(pattern : String)
    email.ilike(pattern)
  end

  def where_ilike_first_name(pattern : String)
    first_name.ilike(pattern)
  end

  def where_ilike_last_name(pattern : String)
    last_name.ilike(pattern)
  end
end
