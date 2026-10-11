class Api::Fs::Formulas::Defaults < ApiAction
  include Api::FsAuth

  post "/api/companies/:company_id/fs/formulas/defaults" do
    if denied = forbid_unless_any!([Permission::ACCOUNTING_WRITE, Permission::SETTINGS_UPDATE], company_id.to_i64)
      return denied
    end

    statement = body_str("statement")
    unless statement && STATEMENTS.includes?(statement)
      response.status_code = 400
      return json({success: false, message: "непознат отчет"})
    end

    mode = body_str("mode") || ""
    existing = FsLineFormulaQuery.new.company_id(company_id).statement(statement)
    count = existing.select_count

    if mode != "fill_empty" && count > 0
      response.status_code = 422
      return json({success: false, message: "вече има формули за този отчет. Изтрийте ги първо."})
    end

    have = Set(String).new
    existing.each { |row| have << row.line_code } if mode == "fill_empty"

    created = 0
    FsCatalog.defaults(statement).each do |code, formula|
      next if have.includes?(code)
      SaveFsLineFormula.create!(
        company_id: company_id.to_i64,
        statement: statement,
        line_code: code,
        formula: formula
      )
      created += 1
    end

    json({success: true, data: {statement: statement, created: created}})
  end
end
