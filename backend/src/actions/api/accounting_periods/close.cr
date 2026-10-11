class Api::AccountingPeriods::Close < ApiAction
  post "/api/companies/:company_id/accounting_periods/:year/:month/close" do
    year = params.get(:year).to_i
    month = params.get(:month).to_i
    notes = params.get?(:notes)

    user = current_user?
    user_id = user.try(&.id)

    # Find or create the period
    period = AccountingPeriodQuery.new
      .company_id(company_id)
      .year(year)
      .month(month)
      .first?

    if period.nil?
      # Create new period
      period = SaveAccountingPeriod.create!(
        company_id: company_id.to_i64,
        year: year,
        month: month,
        status: "closed",
        closed_at: Time.utc,
        closed_by_id: user_id,
        notes: notes
      )
    else
      # Update existing period
      SaveAccountingPeriod.update!(period,
        status: "closed",
        closed_at: Time.utc,
        closed_by_id: user_id,
        notes: notes || period.notes
      )
      period = AccountingPeriodQuery.new.id(period.id).preload_closed_by.first
    end

    json({
      success: true,
      message: "Период #{format_period_name(year, month)} е затворен",
      data:    {
        id:          period.id,
        year:        period.year,
        month:       period.month,
        status:      period.status,
        closed_at:   period.closed_at.try(&.to_s("%Y-%m-%d %H:%M")),
        closed_by:   user.try(&.email),
        notes:       period.notes,
        period_name: format_period_name(year, month),
      },
    })
  end

  private def format_period_name(year : Int32, month : Int32) : String
    months = ["", "Януари", "Февруари", "Март", "Април", "Май", "Юни",
              "Юли", "Август", "Септември", "Октомври", "Ноември", "Декември"]
    "#{months[month]} #{year}"
  end
end
