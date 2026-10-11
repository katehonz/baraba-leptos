class Api::Fs::Formulas::Validate < ApiAction
  include Api::FsAuth

  post "/api/companies/:company_id/fs/formulas/validate" do
    if denied = forbid_unless_any!([Permission::ACCOUNTING_READ, Permission::REPORT_READ], company_id.to_i64)
      return denied
    end

    formula = body_str("formula") || ""
    err = FsFormula.validate(formula)
    if err
      json({success: true, data: {ok: false, error: err, tokens: 0}})
    else
      tokens = FsFormula.parse(formula)
      json({success: true, data: {ok: true, error: "", tokens: tokens.size}})
    end
  end
end
