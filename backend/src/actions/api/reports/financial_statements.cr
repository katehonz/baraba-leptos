class Api::Reports::FinancialStatements < ApiAction
  include Api::FsAuth

  get "/api/companies/:company_id/reports/financial_statements" do
    if denied = forbid_unless_any!([Permission::ACCOUNTING_READ, Permission::REPORT_READ], company_id.to_i64)
      return denied
    end

    kind = params.get?("kind").try(&.to_s) || ""
    statement = KIND_TO_STATEMENT[kind]?
    unless statement
      response.status_code = 400
      return json({success: false, message: "непознат отчет"})
    end

    from_s = params.get?("date_from").try(&.to_s)
    to_s = params.get?("date_to").try(&.to_s)
    if from_s.nil? || from_s.blank? || to_s.nil? || to_s.blank?
      response.status_code = 400
      return json({success: false, message: "липсват дати"})
    end
    date_from = parse_iso_date(from_s)
    date_to = parse_iso_date(to_s)
    unless date_from && date_to
      response.status_code = 422
      return json({success: false, message: "невалидна дата"})
    end
    if date_from > date_to
      response.status_code = 422
      return json({success: false, message: "началната дата е след крайната"})
    end

    include_prior = true
    if raw = params.get?("include_prior")
      case raw.to_s.downcase
      when "0", "false", "no"
        include_prior = false
      when "1", "true", "yes", ""
        include_prior = true
      else
        response.status_code = 422
        return json({success: false, message: "невалиден include_prior"})
      end
    end

    company = CompanyQuery.new.find(company_id)
    result = FsEvaluator.compute(company_id.to_i64, statement, date_from.not_nil!, date_to.not_nil!, include_prior)

    titles = {
      "balance"  => "Счетоводен баланс",
      "pl"       => "Отчет за приходите и разходите",
      "cashflow" => "Отчет за паричните потоци",
      "equity"   => "Отчет за собствения капитал",
    }

    prior_from = Time.utc(date_from.not_nil!.year - 1, date_from.not_nil!.month, 1)
    prior_to = Time.utc(date_to.not_nil!.year - 1, date_to.not_nil!.month, date_to.not_nil!.day)

    json({
      success: true,
      data:    {
        kind:         kind,
        statement:    statement,
        title:        titles[statement],
        company:      {name: company.name, eik: company.eik},
        period:       {
          date_from:  date_from.not_nil!.to_s("%Y-%m-%d"),
          date_to:    date_to.not_nil!.to_s("%Y-%m-%d"),
          prior_from: prior_from.to_s("%Y-%m-%d"),
          prior_to:   prior_to.to_s("%Y-%m-%d"),
        },
        currency:     "EUR",
        rows:         result.rows.map { |r|
          {
            code:              r.code,
            parent:            r.parent || "",
            kind:              r.kind,
            side:              r.side,
            name:              r.name,
            formula:           r.formula,
            current:           r.current,
            prior:             r.prior,
            hidden_in_export:  r.hidden_in_export,
          }
        },
        diagnostics:  result.diagnostics,
        warnings:     result.warnings,
        generated:    Time.utc.to_s("%d.%m.%Y %H:%M"),
      },
    })
  end
end
