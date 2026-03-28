class User < BaseModel
  include Carbon::Emailable
  include Authentic::PasswordAuthenticatable

  table do
    column email : String
    column encrypted_password : String
    column first_name : String?
    column last_name : String?
    column is_active : Bool = true
    column is_super_admin : Bool = false
    column email_verification_token : String?
    column email_verified_at : Time?
    column pending_company_data : String?
    column verification_email_sent_at : Time?

    has_many user_company_roles : UserCompanyRole
  end

  def email_verified? : Bool
    !email_verified_at.nil?
  end

  # Check if can resend verification email (rate limiting - 60 seconds)
  RESEND_COOLDOWN_SECONDS = 60

  def can_resend_verification? : Bool
    return true if verification_email_sent_at.nil?
    Time.utc - verification_email_sent_at.not_nil! > RESEND_COOLDOWN_SECONDS.seconds
  end

  def seconds_until_can_resend : Int32
    return 0 if verification_email_sent_at.nil?
    elapsed = (Time.utc - verification_email_sent_at.not_nil!).total_seconds.to_i
    remaining = RESEND_COOLDOWN_SECONDS - elapsed
    remaining > 0 ? remaining : 0
  end

  # Parse pending company data from JSON
  def parsed_pending_company_data : JSON::Any?
    return nil if pending_company_data.nil? || pending_company_data.not_nil!.empty?
    begin
      JSON.parse(pending_company_data.not_nil!)
    rescue
      nil
    end
  end

  def has_pending_company? : Bool
    !parsed_pending_company_data.nil?
  end

  def super_admin? : Bool
    is_super_admin
  end

  def emailable : Carbon::Address
    Carbon::Address.new(email)
  end

  def full_name : String
    [first_name, last_name].compact.join(" ").strip
  end

  # Get all companies this user has access to
  def companies : Array(Company)
    user_company_roles = UserCompanyRoleQuery.new.user_id(id).preload_company
    user_company_roles.map(&.company)
  end

  # Get role for a specific company
  def role_for_company(company_id : Int64) : Role?
    ucr = UserCompanyRoleQuery.new.user_id(id).company_id(company_id).preload_role.first?
    ucr.try(&.role)
  end

  # Check if user has permission (super admin has all permissions)
  def has_permission?(permission : String, company_id : Int64? = nil) : Bool
    return true if is_super_admin

    if company_id
      role = role_for_company(company_id)
      role.try(&.has_permission?(permission)) || false
    else
      # Check if user has permission in any company
      user_company_roles = UserCompanyRoleQuery.new.user_id(id).preload_role
      user_company_roles.any? { |ucr| ucr.role.has_permission?(permission) }
    end
  end

  # Check if user has any of the required permissions
  def has_any_permission?(permissions : Array(String), company_id : Int64? = nil) : Bool
    return true if is_super_admin
    permissions.any? { |p| has_permission?(p, company_id) }
  end

  # Check if user has all of the required permissions
  def has_all_permissions?(permissions : Array(String), company_id : Int64? = nil) : Bool
    return true if is_super_admin
    permissions.all? { |p| has_permission?(p, company_id) }
  end
end
