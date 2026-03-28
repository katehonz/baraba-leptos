class Api::Reports::AccountLedger < ApiAction
  get "/api/companies/:company_id/reports/account_ledger" do
    date_from = params.get?("date_from")
    date_to = params.get?("date_to")
    account_id_param = params.get?("account_id")
    counterpart_id_param = params.get?("counterpart_id")

    if account_id_param.nil? || account_id_param.empty?
      json({success: false, error: "account_id is required"})
    else
      target_account_id = account_id_param.to_i64
      target_counterpart_id = counterpart_id_param.try(&.to_i64?)

      company = CompanyQuery.new.id(company_id).first
      account = AccountQuery.new.id(target_account_id).first

      # Get all counterparts for lookup
      counterparts_map = {} of Int64 => String
      CounterpartQuery.new.company_id(company_id).each do |cp|
        counterparts_map[cp.id] = cp.name
      end

      # Get all accounts for lookup
      accounts_map = {} of Int64 => {code: String, name: String}
      AccountQuery.new.company_id(company_id).each do |acc|
        accounts_map[acc.id] = {code: acc.code, name: acc.name}
      end

      # 1. Calculate Opening Balance
      # This includes both the official opening balance table AND all transactions before date_from
      opening_debit = 0.0
      opening_credit = 0.0

      # From opening balances table
      ob_query = OpeningBalanceQuery.new.company_id(company_id).account_id(target_account_id)
      if target_counterpart_id
        ob_query = ob_query.counterpart_id(target_counterpart_id)
      end
      ob_query.each do |ob|
        opening_debit += ob.debit
        opening_credit += ob.credit
      end

      # Transactions before date_from
      if date_from && !date_from.empty?
        pre_entries = JournalEntryQuery.new.company_id(company_id).status("posted")
        if target_counterpart_id
          pre_entries = pre_entries.counterpart_id(target_counterpart_id)
        end

        pre_entries.each do |entry|
          entry_date = entry.entry_date.to_s("%Y-%m-%d")
          next if entry_date >= date_from

          lines = parse_json_lines(entry.lines)
          lines.each do |line|
            line_acc_id = line["account_id"]?.try(&.as_i64?) || 0_i64
            next unless line_acc_id == target_account_id

            opening_debit += line["debit"]?.try(&.as_f?) || line["debit_amount"]?.try(&.as_f?) || 0.0
            opening_credit += line["credit"]?.try(&.as_f?) || line["credit_amount"]?.try(&.as_f?) || 0.0
          end
        end
      end

      running_balance = opening_debit - opening_credit

      # 2. Get Transactions in Period
      entries_query = JournalEntryQuery.new.company_id(company_id).status("posted")
      if target_counterpart_id
        entries_query = entries_query.counterpart_id(target_counterpart_id)
      end

      report_rows = [] of NamedTuple(
        date: String,
        document_number: String,
        description: String,
        counterpart_name: String,
        correspondent_account: String,
        debit: Float64,
        credit: Float64,
        balance: Float64
      )

      entries_query.each do |entry|
        entry_date = entry.entry_date.to_s("%Y-%m-%d")

        if date_from && !date_from.empty?
          next if entry_date < date_from
        end

        if date_to && !date_to.empty?
          next if entry_date > date_to
        end

        cp_name = ""
        if cp_id = entry.counterpart_id
          cp_name = counterparts_map[cp_id]? || ""
        end

        lines = parse_json_lines(entry.lines)
        
        # Find lines for our target account
        target_lines = lines.select { |l| (l["account_id"]?.try(&.as_i64?) || 0_i64) == target_account_id }
        
        # Find correspondent accounts (the OTHER accounts in the same entry)
        correspondent_accounts = lines
          .reject { |l| (l["account_id"]?.try(&.as_i64?) || 0_i64) == target_account_id }
          .map { |l| 
            acc_id = l["account_id"]?.try(&.as_i64?) || 0_i64
            accounts_map[acc_id]?.try(&.[:code]) || "???"
          }.uniq.join(", ")

        target_lines.each do |line|
          debit = line["debit"]?.try(&.as_f?) || line["debit_amount"]?.try(&.as_f?) || 0.0
          credit = line["credit"]?.try(&.as_f?) || line["credit_amount"]?.try(&.as_f?) || 0.0
          next if debit == 0.0 && credit == 0.0

          # We don't update running_balance here yet because we need to sort them first
          report_rows << {
            date:                  entry_date,
            document_number:       entry.document_number || "",
            description:           line["description"]?.try(&.as_s?) || entry.description || "",
            counterpart_name:      cp_name,
            correspondent_account: correspondent_accounts,
            debit:                 debit,
            credit:                credit,
            balance:               0.0 # Will fill after sorting
          }
        end
      end

      # Sort by date
      report_rows = report_rows.sort_by(&.[:date])

      # Calculate running balance
      final_rows = report_rows.map do |row|
        running_balance += (row[:debit] - row[:credit])
        row.merge({balance: running_balance})
      end

      json({
        success: true,
        data:    {
          company:   {
            name: company.name,
            eik:  company.eik,
          },
          account:   {
            code: account.code,
            name: account.name,
          },
          counterpart: target_counterpart_id ? {
            id: target_counterpart_id,
            name: counterparts_map[target_counterpart_id]? || ""
          } : nil,
          period:    {
            date_from: date_from,
            date_to:   date_to,
          },
          opening_balance: {
            debit: opening_debit,
            credit: opening_credit,
            total: opening_debit - opening_credit
          },
          rows:      final_rows,
          totals:    {
            debit:  final_rows.sum(&.[:debit]),
            credit: final_rows.sum(&.[:credit]),
            final_balance: running_balance
          },
          generated: Time.utc.to_s("%d.%m.%Y %H:%M"),
        },
      })
    end
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
