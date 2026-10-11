class Api::AccountingPeriods::Check < ApiAction
  # Check if a specific date's period is open
  get "/api/companies/:company_id/accounting_periods/check" do
    date_str = params.get?(:date)

    if date_str.nil?
      response.status_code = 400
      return json({success: false, error: "Date parameter is required"})
    end

    date = Time.parse(date_str, "%Y-%m-%d", Time::Location::UTC)
    year = date.year
    month = date.month

    period = AccountingPeriodQuery.new
      .company_id(company_id)
      .year(year)
      .month(month)
      .first?

    # If no period record exists, it's considered open
    is_open = period.nil? || period.status == "open"

    json({
      success:     true,
      is_open:     is_open,
      year:        year,
      month:       month,
      period_name: format_period_name(year, month),
      status:      period.try(&.status) || "open",
    })
  end

  private def format_period_name(year : Int32, month : Int32) : String
    months = ["", "Януари", "Февруари", "Март", "Април", "Май", "Юни",
              "Юли", "Август", "Септември", "Октомври", "Ноември", "Декември"]
    "#{months[month]} #{year}"
  end
end
