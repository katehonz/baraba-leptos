class Api::Fs::Formulas::Update < ApiAction
  include Api::FsAuth

  put "/api/companies/:company_id/fs/formulas" do
    if denied = forbid_unless_any!([Permission::ACCOUNTING_WRITE, Permission::SETTINGS_UPDATE], company_id.to_i64)
      return denied
    end

    statement = body_str("statement")
    line_code = body_str("line_code")
    formula = body_str("formula") || ""

    unless statement && STATEMENTS.includes?(statement)
      response.status_code = 400
      return json({success: false, message: "непознат отчет"})
    end
    unless line_code && FsCatalog.has_code?(statement, line_code)
      response.status_code = 422
      return json({success: false, message: "непозната статия"})
    end
    if err = FsFormula.validate(formula, statement)
      response.status_code = 422
      return json({success: false, message: err})
    end

    existing = FsLineFormulaQuery.new
      .company_id(company_id.to_i64)
      .statement(statement)
      .line_code(line_code)
      .first?

    saved = if existing
              SaveFsLineFormula.update!(existing, formula: formula)
            else
              SaveFsLineFormula.create!(
                company_id: company_id.to_i64,
                statement: statement,
                line_code: line_code,
                formula: formula
              )
            end

    json({
      success: true,
      data:    {
        id:        saved.id,
        statement: saved.statement,
        line_code: saved.line_code,
        formula:   saved.formula,
      },
    })
  end
end
