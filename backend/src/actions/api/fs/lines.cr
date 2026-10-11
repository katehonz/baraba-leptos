class Api::Fs::Lines < ApiAction
  include Api::FsAuth

  get "/api/companies/:company_id/fs/lines" do
    if denied = forbid_unless_any!([Permission::ACCOUNTING_READ, Permission::REPORT_READ], company_id.to_i64)
      return denied
    end

    statement = params.get?("statement").try(&.to_s)
    if statement.nil? || statement.blank?
      items = [
        {id: "balance", name: "Счетоводен баланс"},
        {id: "pl", name: "Отчет за приходите и разходите"},
        {id: "cashflow", name: "Отчет за паричните потоци"},
        {id: "equity", name: "Отчет за собствения капитал"},
      ]
      return json({success: true, data: {items: items, count: items.size}})
    end

    unless STATEMENTS.includes?(statement)
      response.status_code = 400
      return json({success: false, message: "непознат отчет"})
    end

    items = FsCatalog.lines(statement).map do |row|
      {
        code:   row.code,
        parent: row.parent || "",
        kind:   row.kind,
        side:   row.side,
        name:   row.name,
      }
    end
    json({success: true, data: {statement: statement, count: items.size, items: items}})
  end
end
