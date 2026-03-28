# GET /api/auth/me - Get current user profile
class Api::Auth::Me < ApiAction
  get "/api/auth/me" do
    if user = current_user?
      # Get user's company roles
      user_roles = UserCompanyRoleQuery.new.user_id(user.id).preload_role.preload_company
      company_roles = user_roles.map do |ucr|
        {
          company_id:   ucr.company_id,
          company_name: ucr.company.name,
          role:         ucr.role.name,
        }
      end

      json({
        success: true,
        data:    {
          id:             user.id,
          email:          user.email,
          first_name:     user.first_name,
          last_name:      user.last_name,
          full_name:      user.full_name,
          is_active:      user.is_active,
          is_super_admin: user.is_super_admin,
          company_roles:  company_roles,
          created_at:     user.created_at.to_s("%Y-%m-%d %H:%M"),
        },
      })
    else
      response.status_code = 401
      json({success: false, message: "Не сте влезли в системата"})
    end
  end
end
