class Api::Reports::GeneralLedger < ApiAction
  get "/api/companies/:company_id/reports/general_ledger" do
    date_from = params.get?("date_from")
    date_to = params.get?("date_to")
    account_filter = params.get?("account_filter")

    company = CompanyQuery.new.id(company_id).first

    # Build accounts lookup map, applying prefix filter if present
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

    # Build counterpart lookup
    counterparts_map = {} of Int64 => String
    CounterpartQuery.new.company_id(company_id).each do |cp|
      counterparts_map[cp.id] = cp.name
    end

    # Get opening balances for filtered accounts
    opening_balances = {} of Int64 => {debit: Float64, credit: Float64}
    accounts_map.each_key do |account_id|
      if date_from && !date_from.empty?
        balance = OpeningBalanceQuery.new
          .company_id(company_id.to_i64)
          .account_id(account_id)
          .first?
        if balance
          opening_balances[account_id] = {debit: balance.debit, credit: balance.credit}
        else
          opening_balances[account_id] = {debit: 0.0, credit: 0.0}
        end
      else
        opening_balances[account_id] = {debit: 0.0, credit: 0.0}
      end
    end

    # Single-pass through all posted entries, collect transactions per account
    account_transactions = {} of Int64 => Array(NamedTuple(
      entry_date: String,
      entry_date_sort: String,
      document_number: String,
      description: String,
      counterpart_name: String,
      counterpart_codes: String,
      debit: Float64,
      credit: Float64
    ))

    # Initialize arrays for all accounts
    accounts_map.each_key do |account_id|
      account_transactions[account_id] = [] of NamedTuple(
        entry_date: String,
        entry_date_sort: String,
        document_number: String,
        description: String,
        counterpart_name: String,
        counterpart_codes: String,
        debit: Float64,
        credit: Float64
      )
    end

    entries = JournalEntryQuery.new.company_id(company_id).status("posted")
    entries.each do |entry|
      entry_date = entry.entry_date.to_s("%Y-%m-%d")

      if date_from && !date_from.empty?
        next if entry_date < date_from
      end

      if date_to && !date_to.empty?
        next if entry_date > date_to
      end

      counterpart_name = ""
      if cp_id = entry.counterpart_id
        counterpart_name = counterparts_map[cp_id]? || ""
      end

      lines = parse_json_lines(entry.lines)

      # Collect all account codes in this entry for counterpart lookup
      all_line_codes = {} of Int64 => String
      lines.each do |line|
        lid = line["account_id"]?.try(&.as_i64?) || 0_i64
        lcode = line["account_code"]?.try(&.as_s?) || accounts_map[lid]?.try(&.[:code]) || lid.to_s
        all_line_codes[lid] = lcode
      end

      lines.each do |line|
        line_account_id = line["account_id"]?.try(&.as_i64?) || 0_i64
        next unless accounts_map.has_key?(line_account_id)

        debit = line["debit"]?.try(&.as_f?) || line["debit_amount"]?.try(&.as_f?) || 0.0
        credit = line["credit"]?.try(&.as_f?) || line["credit_amount"]?.try(&.as_f?) || 0.0
        next if debit == 0.0 && credit == 0.0

        line_desc = line["description"]?.try(&.as_s?) || entry.description

        # Find counterpart accounts (all OTHER accounts in this entry)
        counterpart_codes = [] of String
        all_line_codes.each do |other_id, other_code|
          next if other_id == line_account_id
          counterpart_codes << other_code unless counterpart_codes.includes?(other_code)
        end

        account_transactions[line_account_id] << {
          entry_date:       entry.entry_date.to_s("%d.%m.%Y"),
          entry_date_sort:  entry_date,
          document_number:  entry.document_number || "",
          description:      line_desc,
          counterpart_name: counterpart_name,
          counterpart_codes: counterpart_codes.join(", "),
          debit:            debit,
          credit:           credit,
        }
      end
    end

    # Build response: one section per account
    account_sections = [] of NamedTuple(
      account_code: String,
      account_name: String,
      opening_debit: Float64,
      opening_credit: Float64,
      transactions: Array(NamedTuple(
        entry_date: String,
        entry_date_sort: String,
        document_number: String,
        description: String,
        counterpart_name: String,
        counterpart_codes: String,
        debit: Float64,
        credit: Float64
      )),
      turnover_debit: Float64,
      turnover_credit: Float64,
      closing_debit: Float64,
      closing_credit: Float64
    )

    accounts_map.each do |account_id, acc_info|
      transactions = account_transactions[account_id].sort_by(&.[:entry_date_sort])

      opening = opening_balances[account_id]? || {debit: 0.0, credit: 0.0}
      turnover_debit = transactions.sum(&.[:debit])
      turnover_credit = transactions.sum(&.[:credit])

      opening_balance = opening[:debit] - opening[:credit]
      closing_balance = opening_balance + turnover_debit - turnover_credit
      closing_debit = closing_balance > 0 ? closing_balance : 0.0
      closing_credit = closing_balance < 0 ? closing_balance.abs : 0.0

      # Skip accounts with no activity and no opening balance
      next if opening[:debit] == 0.0 &&
              opening[:credit] == 0.0 &&
              transactions.empty?

      account_sections << {
        account_code:    acc_info[:code],
        account_name:    acc_info[:name],
        opening_debit:   opening[:debit],
        opening_credit:  opening[:credit],
        transactions:    transactions,
        turnover_debit:  turnover_debit,
        turnover_credit: turnover_credit,
        closing_debit:   closing_debit,
        closing_credit:  closing_credit,
      }
    end

    # Sort by account code
    account_sections = account_sections.sort_by(&.[:account_code])

    # Grand totals
    grand_totals = {
      opening_debit:   account_sections.sum(&.[:opening_debit]),
      opening_credit:  account_sections.sum(&.[:opening_credit]),
      turnover_debit:  account_sections.sum(&.[:turnover_debit]),
      turnover_credit: account_sections.sum(&.[:turnover_credit]),
      closing_debit:   account_sections.sum(&.[:closing_debit]),
      closing_credit:  account_sections.sum(&.[:closing_credit]),
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
        },
        accounts:  account_sections,
        totals:    grand_totals,
        generated: Time.utc.to_s("%d.%m.%Y %H:%M"),
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
