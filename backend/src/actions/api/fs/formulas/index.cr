class Api::Fs::Formulas::Index < ApiAction
  include Api::FsAuth

  get "/api/companies/:company_id/fs/formulas" do
    if denied = forbid_unless_any!([Permission::ACCOUNTING_READ, Permission::REPORT_READ], company_id.to_i64)
      return denied
    end

    statement = params.get?("statement").try(&.to_s)
    unless statement && STATEMENTS.includes?(statement)
      response.status_code = 400
      return json({success: false, message: "непознат отчет"})
    end

    items = FsLineFormulaQuery.new.company_id(company_id).statement(statement).map do |row|
      {id: row.id, line_code: row.line_code, formula: row.formula}
    end
    json({success: true, data: {statement: statement, count: items.size, items: items}})
  end
end
