class Api::Roles::Index < ApiAction
  get "/api/roles" do
    roles = RoleQuery.new

    json({
      success: true,
      data:    roles.map { |r| role_json(r) },
    })
  end

  private def role_json(role : Role)
    user_count = UserCompanyRoleQuery.new.role_id(role.id).select_count

    {
      id:               role.id,
      name:             role.name,
      description:      role.description,
      permissions:      role.permissions,
      permissions_list: role.permissions_array,
      user_count:       user_count,
      is_super_admin:   role.is_super_admin?,
    }
  end
end
