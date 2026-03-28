class Api::Reports::TrialBalanceByCounterpart < ApiAction
  get "/api/companies/:company_id/reports/trial_balance_by_counterpart" do
    date_from = params.get?("date_from")
    date_to = params.get?("date_to")
    counterpart_id_param = params.get?("counterpart_id")
    account_filter = params.get?("account_filter")

    target_counterpart_id = counterpart_id_param.try(&.to_i64?)

    company = CompanyQuery.new.id(company_id).first

    # Get counterpart name if target_counterpart_id is set
    counterpart_name = ""
    if target_counterpart_id
      counterpart = CounterpartQuery.new.id(target_counterpart_id).first?
      counterpart_name = counterpart.try(&.name) || ""
    end

    # Get all counterparts for lookup
    counterparts_map = {} of Int64 => String
    CounterpartQuery.new.company_id(company_id).each do |cp|
      counterparts_map[cp.id] = cp.name
    end

    # Build accounts lookup map
    filter_prefix = ""
    if account_filter && !account_filter.empty?
      filter_prefix = account_filter.gsub("*", "")
    end

    accounts_map = {} of Int64 => {code: String, name: String}
    AccountQuery.new.company_id(company_id).is_active(true).each do |acc|
      if !filter_prefix.empty?
        next unless acc.code.starts_with?(filter_prefix)
      end
      accounts_map[acc.id] = {code: acc.code, name: acc.name}
    end

    # Get opening balances
    # We need to aggregate by {account_id, counterpart_id}
    opening_balances = {} of {account_id: Int64, counterpart_id: Int64} => {debit: Float64, credit: Float64}

    ob_query = OpeningBalanceQuery.new.company_id(company_id.to_i64)
    if target_counterpart_id
      ob_query = ob_query.counterpart_id(target_counterpart_id)
    end

    ob_query.each do |balance|
      next unless accounts_map.has_key?(balance.account_id)
      cp_id = balance.counterpart_id || 0_i64
      key = {account_id: balance.account_id, counterpart_id: cp_id}
      opening_balances[key] = {debit: balance.debit, credit: balance.credit}
    end

    # Calculate turnovers per {account, counterpart}
    account_turnovers = {} of {account_id: Int64, counterpart_id: Int64} => {debit: Float64, credit: Float64}

    entries_query = JournalEntryQuery.new.company_id(company_id).status("posted")
    if target_counterpart_id
      entries_query = entries_query.counterpart_id(target_counterpart_id)
    end

    entries_query.each do |entry|
      entry_date = entry.entry_date.to_s("%Y-%m-%d")

      if date_from && !date_from.empty?
        next if entry_date < date_from
      end

      if date_to && !date_to.empty?
        next if entry_date > date_to
      end

      cp_id = entry.counterpart_id || 0_i64

      lines = parse_json_lines(entry.lines)
      lines.each do |line|
        line_account_id = line["account_id"]?.try(&.as_i64?) || 0_i64
        next unless accounts_map.has_key?(line_account_id)

        debit = line["debit"]?.try(&.as_f?) || line["debit_amount"]?.try(&.as_f?) || 0.0
        credit = line["credit"]?.try(&.as_f?) || line["credit_amount"]?.try(&.as_f?) || 0.0
        next if debit == 0.0 && credit == 0.0

        key = {account_id: line_account_id, counterpart_id: cp_id}

        if existing = account_turnovers[key]?
          account_turnovers[key] = {
            debit:  existing[:debit] + debit,
            credit: existing[:credit] + credit,
          }
        else
          account_turnovers[key] = {debit: debit, credit: credit}
        end
      end
    end

    # Build report rows
    report_data = [] of NamedTuple(
      account_code: String,
      account_name: String,
      counterpart_id: Int64,
      counterpart_name: String,
      opening_debit: Float64,
      opening_credit: Float64,
      turnover_debit: Float64,
      turnover_credit: Float64,
      closing_debit: Float64,
      closing_credit: Float64
    )

    # Include all {account, counterpart} pairs that have activity
    all_keys = (account_turnovers.keys + opening_balances.keys).uniq

    all_keys.each do |key|
      acc_info = accounts_map[key[:account_id]]?
      next unless acc_info

      cp_name = counterparts_map[key[:counterpart_id]]? || "No Counterpart"

      opening = opening_balances[key]? || {debit: 0.0, credit: 0.0}
      turnover = account_turnovers[key]? || {debit: 0.0, credit: 0.0}

      opening_balance = opening[:debit] - opening[:credit]
      turnover_balance = turnover[:debit] - turnover[:credit]
      closing_balance = opening_balance + turnover_balance

      closing_debit = closing_balance > 0 ? closing_balance : 0.0
      closing_credit = closing_balance < 0 ? closing_balance.abs : 0.0

      report_data << {
        account_code:    acc_info[:code],
        account_name:    acc_info[:name],
        counterpart_id:  key[:counterpart_id],
        counterpart_name: cp_name,
        opening_debit:   opening[:debit],
        opening_credit:  opening[:credit],
        turnover_debit:  turnover[:debit],
        turnover_credit: turnover[:credit],
        closing_debit:   closing_debit,
        closing_credit:  closing_credit,
      }
    end

    # Sort by account code, then counterpart name
    report_data = report_data.sort_by { |r| {r[:account_code], r[:counterpart_name]} }

    # Calculate totals
    totals = {
      opening_debit:   report_data.sum(&.[:opening_debit]),
      opening_credit:  report_data.sum(&.[:opening_credit]),
      turnover_debit:  report_data.sum(&.[:turnover_debit]),
      turnover_credit: report_data.sum(&.[:turnover_credit]),
      closing_debit:   report_data.sum(&.[:closing_debit]),
      closing_credit:  report_data.sum(&.[:closing_credit]),
    }

    json({
      success: true,
      data:    {
        company:          {
          name: company.name,
          eik:  company.eik,
        },
        counterpart:      target_counterpart_id ? {
          id:   target_counterpart_id,
          name: counterpart_name,
        } : nil,
        period:           {
          date_from:      date_from,
          date_to:        date_to,
          account_filter: account_filter,
        },
        rows:             report_data,
        totals:           totals,
        generated:        Time.utc.to_s("%d.%m.%Y %H:%M"),
      },
    })
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
