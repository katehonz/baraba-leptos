# Admin statistics endpoint for dashboard
class Api::Admin::Stats < ApiAction
  include Api::Auth::RequireAuthToken

  get "/api/admin/stats" do
    # Authorization: require super_admin permissions
    user = current_user?.not_nil!
    unless user.is_super_admin
      response.status_code = 403
      return json({success: false, message: "Само супер администратори имат достъп"})
    end

    json({
      success: true,
      data:    {
        users:               user_stats,
        companies:           company_stats,
        roles:               role_stats,
        recent_users:        recent_users,
        recent_companies:    recent_companies,
      },
    })
  end

  private def user_stats
    {
      total:         UserQuery.new.select_count,
      active:        UserQuery.new.is_active(true).select_count,
      inactive:      UserQuery.new.is_active(false).select_count,
      super_admins:  UserQuery.new.is_super_admin(true).select_count,
      verified:      UserQuery.new.email_verified_at.is_not_nil.select_count,
      unverified:    UserQuery.new.email_verified_at.is_nil.select_count,
    }
  end

  private def company_stats
    {
      total: CompanyQuery.new.select_count,
    }
  end

  private def role_stats
    roles = RoleQuery.new.to_a
    roles.map do |role|
      user_count = UserCompanyRoleQuery.new.role_id(role.id).select_count
      {
        id:          role.id,
        name:        role.name,
        description: role.description,
        user_count:  user_count,
      }
    end
  end

  private def recent_users(limit = 10)
    users = UserQuery.new.created_at.desc_order.limit(limit).to_a
    users.map do |user|
      {
        id:         user.id,
        email:      user.email,
        first_name: user.first_name,
        last_name:  user.last_name,
        is_active:  user.is_active,
        created_at: user.created_at,
      }
    end
  end

  private def recent_companies(limit = 10)
    companies = CompanyQuery.new.created_at.desc_order.limit(limit).to_a
    companies.map do |company|
      {
        id:         company.id,
        name:       company.name,
        eik:        company.eik,
        city:       company.city,
        created_at: company.created_at,
      }
    end
  end
end
