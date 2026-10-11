class Api::Fs::Formulas::Clear < ApiAction
  include Api::FsAuth

  post "/api/companies/:company_id/fs/formulas/clear" do
    if denied = forbid_unless_any!([Permission::ACCOUNTING_WRITE, Permission::SETTINGS_UPDATE], company_id.to_i64)
      return denied
    end

    statement = body_str("statement")
    unless statement && STATEMENTS.includes?(statement)
      response.status_code = 422
      return json({success: false, message: "непознат отчет"})
    end

    deleted = 0
    FsLineFormulaQuery.new.company_id(company_id).statement(statement).each do |row|
      row.delete
      deleted += 1
    end
    json({success: true, data: {statement: statement, deleted: deleted}})
  end
end
