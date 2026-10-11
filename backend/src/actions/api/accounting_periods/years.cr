class Api::AccountingPeriods::Years < ApiAction
  # Get all years that have periods or transactions
  get "/api/companies/:company_id/accounting_periods/years" do
    # Get years from accounting periods
    period_years = AccountingPeriodQuery.new
      .company_id(company_id)
      .distinct_on(&.year)
      .map(&.year)

    # Get years from journal entries
    entry_years = JournalEntryQuery.new
      .company_id(company_id)
      .map { |e| e.entry_date.year }
      .uniq

    # Combine and sort
    all_years = (period_years + entry_years).uniq.sort.reverse

    # Add current year if not present
    current_year = Time.local.year
    all_years = ([current_year] + all_years).uniq.sort.reverse

    json({
      success: true,
      data:    all_years,
    })
  end
end
