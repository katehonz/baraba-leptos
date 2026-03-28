# Get user's company roles
class Api::Users::CompanyRoles::Index < ApiAction
  include Api::Auth::RequireAuthToken

  get "/api/users/:user_id/company_roles" do
    current = current_user?.not_nil!

    unless current.is_super_admin
      response.status_code = 403
      return json({success: false, message: "Нямате достъп"})
    end

    user = UserQuery.new.id(user_id).first?

    if user.nil?
      json({success: false, message: "Потребителят не е намерен"})
    else
      roles = UserCompanyRoleQuery.new
        .user_id(user_id)
        .preload_company
        .preload_role

      json({
        success: true,
        data:    roles.map { |ucr| {
          company_id:   ucr.company.id,
          company_name: ucr.company.name,
          role_id:      ucr.role.id,
          role:         ucr.role.name,
        } },
      })
    end
  end
end

# Assign company role to user
class Api::Users::CompanyRoles::Create < ApiAction
  include Api::Auth::RequireAuthToken

  post "/api/users/:user_id/company_roles" do
    current = current_user?.not_nil!

    unless current.is_super_admin
      response.status_code = 403
      return json({success: false, message: "Нямате достъп"})
    end

    user = UserQuery.new.id(user_id).first?
    company_id = params.get?(:company_id).try(&.to_i64?)
    role_name = params.get?(:role).to_s

    if user.nil?
      json({success: false, message: "Потребителят не е намерен"})
    elsif company_id.nil?
      json({success: false, message: "Фирмата е задължителна"})
    elsif role_name.empty?
      json({success: false, message: "Ролята е задължителна"})
    else
      company = CompanyQuery.new.id(company_id).first?
      role = RoleQuery.new.name(role_name).first?

      if company.nil?
        json({success: false, message: "Фирмата не е намерена"})
      elsif role.nil?
        json({success: false, message: "Ролята не е намерена"})
      else
        # Check if already exists
        existing = UserCompanyRoleQuery.new
          .user_id(user_id)
          .company_id(company_id)
          .first?

        if existing
          # Update role
          SaveUserCompanyRole.update!(existing, role_id: role.id)
          json({
            success: true,
            message: "Ролята е обновена",
          })
        else
          # Create new
          SaveUserCompanyRole.create!(
            user_id: user.id,
            company_id: company.id,
            role_id: role.id
          )
          json({
            success: true,
            message: "Достъпът е добавен",
          })
        end
      end
    end
  end
end

# Remove company role from user
class Api::Users::CompanyRoles::Delete < ApiAction
  include Api::Auth::RequireAuthToken

  delete "/api/users/:user_id/company_roles/:company_id" do
    current = current_user?.not_nil!

    unless current.is_super_admin
      response.status_code = 403
      return json({success: false, message: "Нямате достъп"})
    end

    ucr = UserCompanyRoleQuery.new
      .user_id(user_id)
      .company_id(company_id)
      .first?

    if ucr
      ucr.delete
      json({success: true, message: "Достъпът е премахнат"})
    else
      json({success: false, message: "Връзката не е намерена"})
    end
  end
end
