class Api::AccountingPeriods::Reopen < ApiAction
  post "/api/companies/:company_id/accounting_periods/:year/:month/reopen" do
    year = params.get(:year).to_i
    month = params.get(:month).to_i

    period = AccountingPeriodQuery.new
      .company_id(company_id)
      .year(year)
      .month(month)
      .first?

    if period.nil?
      response.status_code = 404
      return json({success: false, error: "Периодът не съществува"})
    end

    if period.status == "open"
      return json({
        success: true,
        message: "Периодът вече е отворен",
        data:    period_json(period),
      })
    end

    SaveAccountingPeriod.update!(period,
      status: "open",
      closed_at: nil,
      closed_by_id: nil
    )

    period = AccountingPeriodQuery.new.id(period.id).first

    json({
      success: true,
      message: "Период #{format_period_name(year, month)} е отворен отново",
      data:    period_json(period),
    })
  end

  private def period_json(period : AccountingPeriod)
    {
      id:          period.id,
      year:        period.year,
      month:       period.month,
      status:      period.status,
      closed_at:   period.closed_at.try(&.to_s("%Y-%m-%d %H:%M")),
      notes:       period.notes,
      period_name: format_period_name(period.year, period.month),
    }
  end

  private def format_period_name(year : Int32, month : Int32) : String
    months = ["", "Януари", "Февруари", "Март", "Април", "Май", "Юни",
              "Юли", "Август", "Септември", "Октомври", "Ноември", "Декември"]
    "#{months[month]} #{year}"
  end
end
