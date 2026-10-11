# Tenant isolation: requests to company-scoped routes (/api/companies/:company_id/...
# or /api/companies/:id/...) require an authenticated user who is a super admin,
# the company owner, or has a role in that company.
module Api::Auth::RequireCompanyAccess
  macro included
    before require_company_access
  end

  private def require_company_access : Lucky::ActionPipes::Continue | Lucky::Response
    return continue unless context.request.path.starts_with?("/api/companies/")

    raw = params.get?(:company_id) || params.get?(:id)
    return continue if raw.nil? || raw.to_s.empty?

    cid = raw.to_i64?
    unless cid
      response.status_code = 400
      return json({success: false, message: "Невалидна фирма"})
    end

    user = current_user?
    unless user
      response.status_code = 401
      return json({success: false, message: "Не сте автентикирани"})
    end

    return continue if user.super_admin?

    company = CompanyQuery.new.id(cid).first?
    unless company
      response.status_code = 404
      return json({success: false, message: "Фирмата не е намерена"})
    end

    return continue if company.owner_id == user.id

    if UserCompanyRoleQuery.new.user_id(user.id).company_id(cid).select_count > 0
      continue
    else
      AuditLogger.log(context, "company_access_denied", user.email, "company_id=#{cid}")
      response.status_code = 403
      json({success: false, message: "Нямате достъп до тази фирма"})
    end
  end
end
