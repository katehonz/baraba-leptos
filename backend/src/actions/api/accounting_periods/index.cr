class Api::AccountingPeriods::Index < ApiAction
  get "/api/companies/:company_id/accounting_periods" do
    year_filter = params.get?(:year)

    query = AccountingPeriodQuery.new
      .company_id(company_id)
      .preload_closed_by

    if year_filter
      query = query.year(year_filter.to_i)
    end

    periods = query.year.desc_order.month.desc_order

    json({
      success: true,
      data:    periods.map { |p| period_json(p) },
    })
  end

  private def period_json(period : AccountingPeriod)
    {
      id:            period.id,
      year:          period.year,
      month:         period.month,
      status:        period.status,
      closed_at:     period.closed_at.try(&.to_s("%Y-%m-%d %H:%M")),
      closed_by:     period.closed_by.try { |u| {id: u.id, email: u.email} },
      notes:         period.notes,
      period_name:   format_period_name(period.year, period.month),
    }
  end

  private def format_period_name(year : Int32, month : Int32) : String
    months = ["", "Януари", "Февруари", "Март", "Април", "Май", "Юни",
              "Юли", "Август", "Септември", "Октомври", "Ноември", "Декември"]
    "#{months[month]} #{year}"
  end
end
