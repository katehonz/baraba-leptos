# Role - роля с права
# Migrated from old Java Role model
class Role < BaseModel
  table do
    column name : String                # e.g., "accountant", "admin", "viewer"
    column description : String?
    column permissions : String?        # JSON array of permission codes

    has_many user_company_roles : UserCompanyRole
  end

  # Parse permissions JSON to array
  def permissions_array : Array(String)
    return [] of String if permissions.nil? || permissions.try(&.empty?)
    begin
      JSON.parse(permissions.not_nil!).as_a.map(&.as_s)
    rescue
      [] of String
    end
  end

  # Check if role has a specific permission
  def has_permission?(required : String) : Bool
    perms = permissions_array
    Permission.has_permission?(perms, required)
  end

  # Check if role has all of the required permissions
  def has_all_permissions?(required : Array(String)) : Bool
    perms = permissions_array
    Permission.has_all_permissions?(perms, required)
  end

  # Check if role has any of the required permissions
  def has_any_permission?(required : Array(String)) : Bool
    perms = permissions_array
    Permission.has_any_permission?(perms, required)
  end

  # Check if this is a super admin role (has all permissions)
  def is_super_admin? : Bool
    permissions_array.includes?(Permission::ALL)
  end
end
