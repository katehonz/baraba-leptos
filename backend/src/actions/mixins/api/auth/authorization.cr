# Authorization mixin for checking user permissions
# Include this module in API actions that need permission checks
module Api::Auth::Authorization
  # Check if user has the required permission
  # Raises 403 Forbidden if permission check fails
  private def authorize!(permission : String, company_id : Int64? = nil) : Nil
    user = current_user?
    unless user
      raise_unauthorized
    end

    unless user.has_permission?(permission, company_id)
      raise_forbidden("Нямате права за това действие")
    end
  end

  # Check if user has any of the required permissions
  private def authorize_any!(permissions : Array(String), company_id : Int64? = nil) : Nil
    user = current_user?
    unless user
      raise_unauthorized
    end

    unless user.has_any_permission?(permissions, company_id)
      raise_forbidden("Нямате права за това действие")
    end
  end

  # Check if user has all of the required permissions
  private def authorize_all!(permissions : Array(String), company_id : Int64? = nil) : Nil
    user = current_user?
    unless user
      raise_unauthorized
    end

    unless user.has_all_permissions?(permissions, company_id)
      raise_forbidden("Нямате права за това действие")
    end
  end

  # Check if current user is super admin
  private def require_super_admin! : Nil
    user = current_user?
    unless user
      raise_unauthorized
    end

    unless user.super_admin?
      raise_forbidden("Само супер администратор има достъп")
    end
  end

  # Check if current user is admin (super admin or company admin)
  private def require_admin!(company_id : Int64? = nil) : Nil
    user = current_user?
    unless user
      raise_unauthorized
    end

    return if user.super_admin?

    if company_id
      role = user.role_for_company(company_id)
      unless role && role.name == "admin"
        raise_forbidden("Нямате административен достъп")
      end
    else
      # Check if user is admin in any company
      roles = UserCompanyRoleQuery.new.user_id(user.id).preload_role
      unless roles.any? { |ucr| ucr.role.name == "admin" }
        raise_forbidden("Нямате административен достъп")
      end
    end
  end

  # Get current company ID from params or header
  private def current_company_id : Int64?
    params.get?(:company_id).try(&.to_i64) ||
      context.request.headers["X-Company-ID"]?.try(&.to_i64)
  end

  # Get current company
  private def current_company : Company?
    company_id = current_company_id
    return nil unless company_id
    CompanyQuery.new.find(company_id)
  rescue
    nil
  end

  private def raise_unauthorized : NoReturn
    raise AuthorizationError::Unauthorized.new("Не сте автентикирани")
  end

  private def raise_forbidden(message : String = "Достъпът е забранен") : NoReturn
    raise AuthorizationError::Forbidden.new(message)
  end
end

# Custom error classes for authorization
module AuthorizationError
  class Unauthorized < Exception
  end

  class Forbidden < Exception
  end
end
