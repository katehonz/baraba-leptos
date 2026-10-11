# GET /api/companies/:company_id/users - Get users for a specific company
class Api::Companies::Users < ApiAction
  include Api::Auth::RequireAuthToken

  get "/api/companies/:company_id/users" do
    company_id = params.get(:company_id).to_i64
    user = current_user?.not_nil!

    # Check if user has access to this company
    unless user.is_super_admin
      user_role = UserCompanyRoleQuery.new
        .user_id(user.id)
        .company_id(company_id)
        .first?

      unless user_role
        response.status_code = 403
        return json({success: false, message: "Нямате достъп до тази фирма"})
      end
    end

    # Get all users in this company
    user_company_roles = UserCompanyRoleQuery.new
      .company_id(company_id)
      .preload_user
      .preload_role

    users_data = user_company_roles.map do |ucr|
      {
        id:             ucr.user.id,
        email:          ucr.user.email,
        first_name:     ucr.user.first_name,
        last_name:      ucr.user.last_name,
        full_name:      ucr.user.full_name,
        is_active:      ucr.user.is_active,
        is_super_admin: ucr.user.is_super_admin,
        role:           ucr.role.name,
        role_id:        ucr.role.id,
        created_at:     ucr.user.created_at,
      }
    end

    json({
      success: true,
      data:    users_data,
    })
  end
end

# POST /api/companies/:company_id/users - Add user to company
class Api::Companies::AddUser < ApiAction
  include Api::Auth::RequireAuthToken

  post "/api/companies/:company_id/users" do
    company_id = params.get(:company_id).to_i64
    user = current_user?.not_nil!

    # Get the role to assign
    role_name = params.from_json["role"]?.try(&.as_s) || ""
    user_email = params.from_json["email"]?.try(&.as_s) || ""

    # Find the role
    role = RoleQuery.new.name(role_name).first?
    unless role
      response.status_code = 400
      return json({success: false, message: "Невалидна роля: #{role_name}"})
    end

    # Check role hierarchy - non-super_admins can only assign accountant or observer
    unless user.is_super_admin
      user_role = UserCompanyRoleQuery.new
        .user_id(user.id)
        .company_id(company_id)
        .preload_role
        .first?

      unless user_role
        response.status_code = 403
        return json({success: false, message: "Нямате достъп до тази фирма"})
      end

      # Check if user is admin for this company
      unless user_role.role.name == "admin"
        response.status_code = 403
        return json({success: false, message: "Само администратори могат да добавят потребители"})
      end

      # Admin can only assign accountant or observer roles
      allowed_roles = ["accountant", "observer"]
      unless allowed_roles.includes?(role_name)
        response.status_code = 403
        return json({success: false, message: "Можете да добавяте само счетоводители и наблюдатели"})
      end
    end

    # Find or create user
    target_user = UserQuery.new.email(user_email).first?

    unless target_user
      response.status_code = 404
      return json({success: false, message: "Потребителят не е намерен. Първо трябва да се регистрира."})
    end

    # Check if user already has a role in this company
    existing_role = UserCompanyRoleQuery.new
      .user_id(target_user.id)
      .company_id(company_id)
      .first?

    if existing_role
      # Update existing role
      SaveUserCompanyRole.update!(existing_role, role_id: role.id)
    else
      # Create new role assignment
      SaveUserCompanyRole.create!(
        user_id: target_user.id,
        company_id: company_id,
        role_id: role.id
      )
    end

    json({
      success: true,
      message: "Потребителят е добавен успешно",
    })
  end
end

# DELETE /api/companies/:company_id/users/:user_id - Remove user from company
class Api::Companies::RemoveUser < ApiAction
  include Api::Auth::RequireAuthToken

  delete "/api/companies/:company_id/users/:user_id" do
    company_id = params.get(:company_id).to_i64
    target_user_id = params.get(:user_id).to_i64
    user = current_user?.not_nil!

    # Cannot remove yourself
    if target_user_id == user.id
      response.status_code = 400
      return json({success: false, message: "Не можете да премахнете себе си"})
    end

    # Check permissions
    unless user.is_super_admin
      user_role = UserCompanyRoleQuery.new
        .user_id(user.id)
        .company_id(company_id)
        .preload_role
        .first?

      unless user_role && user_role.role.name == "admin"
        response.status_code = 403
        return json({success: false, message: "Само администратори могат да премахват потребители"})
      end

      # Check target user's role - admin can only remove accountant/observer
      target_role = UserCompanyRoleQuery.new
        .user_id(target_user_id)
        .company_id(company_id)
        .preload_role
        .first?

      if target_role
        unless ["accountant", "observer"].includes?(target_role.role.name)
          response.status_code = 403
          return json({success: false, message: "Не можете да премахвате администратори"})
        end
      end
    end

    # Remove the user from company
    user_company_role = UserCompanyRoleQuery.new
      .user_id(target_user_id)
      .company_id(company_id)
      .first?

    if user_company_role
      user_company_role.delete
      json({success: true, message: "Потребителят е премахнат от фирмата"})
    else
      response.status_code = 404
      json({success: false, message: "Потребителят не е намерен в тази фирма"})
    end
  end
end
