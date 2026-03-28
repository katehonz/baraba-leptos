class Api::Reports::TrialBalance < ApiAction
  get "/api/companies/:company_id/reports/trial_balance" do
    date_from = params.get?("date_from")
    date_to = params.get?("date_to")
    account_filter = params.get?("account_filter")
    group_by_level = params.get?("group_by_level").try(&.to_i?)

    # Get all active accounts for the company
    all_accounts = AccountQuery.new.company_id(company_id).is_active(true)

    # Apply account filter (e.g., "3*" -> all accounts starting with "3")
    filter_prefix = ""
    if account_filter && !account_filter.empty?
      filter_prefix = account_filter.gsub("*", "")
    end

    filtered_accounts = if !filter_prefix.empty?
      all_accounts.to_a.select { |a| a.code.starts_with?(filter_prefix) }
    else
      all_accounts.to_a
    end

    # Get company info
    company = CompanyQuery.new.id(company_id).first

    # Build report data
    report_data = filtered_accounts.map do |account|
      # Calculate turnovers from journal lines (from JSON in journal_entries)
      turnover = calculate_account_turnover(account.id, company_id.to_i64, date_from, date_to)

      # Get opening balance
      opening = get_opening_balance(account.id, company_id.to_i64, date_from)

      {
        account_code:    account.code,
        account_name:    account.name,
        opening_debit:   opening[:debit],
        opening_credit:  opening[:credit],
        turnover_debit:  turnover[:debit],
        turnover_credit: turnover[:credit],
      }
    end

    # Aggregate if level is specified
    if group_by_level && group_by_level > 0
      aggregated = {} of String => {
        account_code: String,
        account_name: String,
        opening_debit: Float64,
        opening_credit: Float64,
        turnover_debit: Float64,
        turnover_credit: Float64,
      }

      report_data.each do |row|
        prefix = row[:account_code][0...group_by_level]
        if existing = aggregated[prefix]?
          aggregated[prefix] = {
            account_code:    prefix,
            account_name:    existing[:account_name], # Keep first name or we could try to find a parent account name
            opening_debit:   existing[:opening_debit] + row[:opening_debit],
            opening_credit:  existing[:opening_credit] + row[:opening_credit],
            turnover_debit:  existing[:turnover_debit] + row[:turnover_debit],
            turnover_credit: existing[:turnover_credit] + row[:turnover_credit],
          }
        else
          # Try to find a descriptive name for the synthetic account
          synthetic_name = row[:account_name]
          if row[:account_code].size > group_by_level
             # potentially look up the parent account name if it exists
             parent = AccountQuery.new.company_id(company_id).code(prefix).first?
             synthetic_name = parent.name if parent
          end

          aggregated[prefix] = {
            account_code:    prefix,
            account_name:    synthetic_name,
            opening_debit:   row[:opening_debit],
            opening_credit:  row[:opening_credit],
            turnover_debit:  row[:turnover_debit],
            turnover_credit: row[:turnover_credit],
          }
        end
      end
      report_data = aggregated.values.sort_by(&.[:account_code])
    end

    # Final calculation of closing balances for each row
    report_rows = report_data.map do |row|
      opening_balance = row[:opening_debit] - row[:opening_credit]
      turnover_balance = row[:turnover_debit] - row[:turnover_credit]
      closing_balance = opening_balance + turnover_balance

      closing_debit = closing_balance > 0 ? closing_balance : 0.0
      closing_credit = closing_balance < 0 ? closing_balance.abs : 0.0

      {
        account_code:    row[:account_code],
        account_name:    row[:account_name],
        opening_debit:   row[:opening_debit],
        opening_credit:  row[:opening_credit],
        turnover_debit:  row[:turnover_debit],
        turnover_credit: row[:turnover_credit],
        closing_debit:   closing_debit,
        closing_credit:  closing_credit,
      }
    end

    # Filter out accounts with no activity
    report_rows = report_rows.reject do |row|
      row[:opening_debit] == 0.0 &&
        row[:opening_credit] == 0.0 &&
        row[:turnover_debit] == 0.0 &&
        row[:turnover_credit] == 0.0
    end

    # Calculate totals
    totals = {
      opening_debit:   report_rows.sum(&.[:opening_debit]),
      opening_credit:  report_rows.sum(&.[:opening_credit]),
      turnover_debit:  report_rows.sum(&.[:turnover_debit]),
      turnover_credit: report_rows.sum(&.[:turnover_credit]),
      closing_debit:   report_rows.sum(&.[:closing_debit]),
      closing_credit:  report_rows.sum(&.[:closing_credit]),
    }

    json({
      success: true,
      data:    {
        company:   {
          name: company.name,
          eik:  company.eik,
        },
        period:    {
          date_from:      date_from,
          date_to:        date_to,
          account_filter: account_filter,
          group_by_level: group_by_level,
        },
        rows:      report_rows,
        totals:    totals,
        generated: Time.utc.to_s("%d.%m.%Y %H:%M"),
      },
    })
  end

  private def calculate_account_turnover(account_id : Int64, company_id : Int64, date_from : String?, date_to : String?) : NamedTuple(debit: Float64, credit: Float64)
    total_debit = 0.0
    total_credit = 0.0

    # Get all journal entries for the company in the period
    entries = JournalEntryQuery.new.company_id(company_id).status("posted")

    entries.each do |entry|
      # Check date range
      entry_date = entry.entry_date.to_s("%Y-%m-%d")

      if date_from && !date_from.empty?
        next if entry_date < date_from
      end

      if date_to && !date_to.empty?
        next if entry_date > date_to
      end

      # Parse lines from JSON
      lines = parse_json_lines(entry.lines)
      lines.each do |line|
        line_account_id = line["account_id"]?.try(&.as_i64?) || 0_i64

        if line_account_id == account_id
          debit = line["debit"]?.try(&.as_f?) || line["debit_amount"]?.try(&.as_f?) || 0.0
          credit = line["credit"]?.try(&.as_f?) || line["credit_amount"]?.try(&.as_f?) || 0.0
          total_debit += debit
          total_credit += credit
        end
      end
    end

    {debit: total_debit, credit: total_credit}
  end

  private def get_opening_balance(account_id : Int64, company_id : Int64, date_from : String?) : NamedTuple(debit: Float64, credit: Float64)
    # Try to get from opening_balances table
    if date_from && !date_from.empty?
      balance = OpeningBalanceQuery.new
        .company_id(company_id)
        .account_id(account_id)
        .first?

      if balance
        return {debit: balance.debit, credit: balance.credit}
      end
    end

    {debit: 0.0, credit: 0.0}
  end

  private def parse_json_lines(lines_str : String?) : Array(JSON::Any)
    return [] of JSON::Any if lines_str.nil? || lines_str.empty?

    begin
      parsed = JSON.parse(lines_str)
      parsed.as_a? || [] of JSON::Any
    rescue
      [] of JSON::Any
    end
  end
end
