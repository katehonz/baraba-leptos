module Api::FsAuth
  STATEMENTS = FsCatalog::STATEMENTS

  KIND_TO_STATEMENT = {
    "balance_sheet"    => "balance",
    "income_statement" => "pl",
    "cash_flow"        => "cashflow",
    "equity"           => "equity",
  }

  private def forbid_unless_any!(perms : Array(String), company_id : Int64) : Lucky::Response?
    user = current_user?
    unless user
      response.status_code = 401
      return json({success: false, message: "Не сте автентикирани"})
    end
    unless user.has_any_permission?(perms, company_id)
      response.status_code = 403
      return json({success: false, message: "Нямате права за това действие"})
    end
    nil
  end

  private def parse_iso_date(raw : String?) : Time?
    return nil if raw.nil? || raw.blank?
    Time.parse(raw, "%Y-%m-%d", Time::Location::UTC)
  rescue
    nil
  end

  private def json_body : JSON::Any
    params.from_json
  rescue
    JSON::Any.new({} of String => JSON::Any)
  end

  private def body_str(key : String) : String?
    val = json_body[key]?
    return nil if val.nil? || val.raw.nil?
    s = val.as_s?
    return nil if s.nil? || s.blank?
    s.strip
  end
end
