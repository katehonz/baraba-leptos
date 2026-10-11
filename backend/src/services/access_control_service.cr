# AccessControlService - сервиз за контрол на достъпа
# Migrated from old Java AccessControlService
class AccessControlService
  getter user : User

  def initialize(@user : User)
  end

  # Check if user has global super admin access
  def is_super_admin? : Bool
    # Use the explicit database field - only manually promoted users are super admins
    user.is_super_admin
  end

  # Check if user has access to a specific company
  def has_company_access?(company_id : Int64, permission : String) : Bool
    # Super admins have access to everything
    return true if is_super_admin?

    # Find user's role in this company
    user_company_role = UserCompanyRoleQuery.new
      .user_id(user.id)
      .company_id(company_id)
      .preload_role
      .first?

    return false if user_company_role.nil?

    # Check if role has the required permission
    user_company_role.role.has_permission?(permission)
  end

  # Check if user is admin of a specific company
  def is_company_admin?(company_id : Int64) : Bool
    return true if is_super_admin?

    user_company_role = UserCompanyRoleQuery.new
      .user_id(user.id)
      .company_id(company_id)
      .preload_role
      .first?

    return false if user_company_role.nil?

    # Admin has all admin permissions
    user_company_role.role.has_all_permissions?(Permission::ADMIN_ALL)
  end

  # Get user's role in a specific company
  def get_company_role(company_id : Int64) : Role?
    user_company_role = UserCompanyRoleQuery.new
      .user_id(user.id)
      .company_id(company_id)
      .preload_role
      .first?

    user_company_role.try(&.role)
  end

  # Get all companies user has access to
  def accessible_companies : Array(Company)
    if is_super_admin?
      # Super admins can access all companies
      CompanyQuery.new.to_a
    else
      # Get companies through user_company_roles
      user_company_roles = UserCompanyRoleQuery.new.user_id(user.id).preload_company
      user_company_roles.map(&.company)
    end
  end

  # Check multiple permissions at once
  def has_any_company_permission?(company_id : Int64, permissions : Array(String)) : Bool
    return true if is_super_admin?

    user_company_role = UserCompanyRoleQuery.new
      .user_id(user.id)
      .company_id(company_id)
      .preload_role
      .first?

    return false if user_company_role.nil?

    user_company_role.role.has_any_permission?(permissions)
  end

  # Check if user has all required permissions
  def has_all_company_permissions?(company_id : Int64, permissions : Array(String)) : Bool
    return true if is_super_admin?

    user_company_role = UserCompanyRoleQuery.new
      .user_id(user.id)
      .company_id(company_id)
      .preload_role
      .first?

    return false if user_company_role.nil?

    user_company_role.role.has_all_permissions?(permissions)
  end

  # Static helper for quick permission checks
  def self.check(user : User, company_id : Int64, permission : String) : Bool
    new(user).has_company_access?(company_id, permission)
  end
end
