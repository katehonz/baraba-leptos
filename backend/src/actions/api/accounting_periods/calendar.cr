class Api::AccountingPeriods::Calendar < ApiAction
  # Get a full calendar view of all 12 months for a year
  get "/api/companies/:company_id/accounting_periods/calendar/:year" do
    year = params.get(:year).to_i

    # Get all periods for this year
    periods = AccountingPeriodQuery.new
      .company_id(company_id)
      .year(year)
      .preload_closed_by

    # Build a map of month -> period
    period_map = {} of Int32 => AccountingPeriod
    periods.each { |p| period_map[p.month] = p }

    # Generate all 12 months
    months_data = (1..12).map do |month|
      period = period_map[month]?

      {
        year:        year,
        month:       month,
        month_name:  get_month_name(month),
        status:      period.try(&.status) || "open",
        is_closed:   period.try(&.status) == "closed",
        closed_at:   period.try(&.closed_at).try(&.to_s("%Y-%m-%d %H:%M")),
        closed_by:   period.try(&.closed_by).try { |u| u.email },
        notes:       period.try(&.notes),
        has_record:  !period.nil?,
      }
    end

    # Calculate summary
    closed_count = months_data.count { |m| m[:is_closed] }

    json({
      success:       true,
      year:          year,
      months:        months_data,
      closed_count:  closed_count,
      open_count:    12 - closed_count,
    })
  end

  private def get_month_name(month : Int32) : String
    months = ["", "Януари", "Февруари", "Март", "Април", "Май", "Юни",
              "Юли", "Август", "Септември", "Октомври", "Ноември", "Декември"]
    months[month]
  end
end
