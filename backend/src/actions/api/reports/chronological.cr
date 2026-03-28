class Api::Reports::Chronological < ApiAction
  get "/api/companies/:company_id/reports/chronological" do
    date_from = params.get?("date_from")
    date_to = params.get?("date_to")
    account_code = params.get?("account_code")
    counterpart_id_param = params.get?("counterpart_id")
    target_counterpart_id = counterpart_id_param.try(&.to_i64?)

    # Get company info
    company = CompanyQuery.new.id(company_id).first

    # Get all accounts for lookup
    accounts_map = {} of Int64 => {code: String, name: String}
    AccountQuery.new.company_id(company_id).each do |acc|
      accounts_map[acc.id] = {code: acc.code, name: acc.name}
    end

    # Get all counterparts for lookup
    counterparts_map = {} of Int64 => String
    CounterpartQuery.new.company_id(company_id).each do |cp|
      counterparts_map[cp.id] = cp.name
    end

    # Get journal entries
    entries_query = JournalEntryQuery.new.company_id(company_id).status("posted")
    if target_counterpart_id
      entries_query = entries_query.counterpart_id(target_counterpart_id)
    end
    entries = entries_query.to_a

    report_rows = [] of NamedTuple(
      row_number: Int32,
      entry_id: Int64,
      entry_date: String,
      document_number: String,
      description: String,
      counterpart_name: String,
      debit_account: String,
      credit_account: String,
      debit_amount: Float64,
      credit_amount: Float64
    )

    row_num = 0

    entries.each do |entry|
      # Check date range
      entry_date = entry.entry_date.to_s("%Y-%m-%d")
      entry_date_display = entry.entry_date.to_s("%d.%m.%Y")

      if date_from && !date_from.empty?
        next if entry_date < date_from
      end

      if date_to && !date_to.empty?
        next if entry_date > date_to
      end

      # Get counterpart name
      counterpart_name = ""
      if cp_id = entry.counterpart_id
        counterpart_name = counterparts_map[cp_id]? || ""
      end

      # Parse lines from JSON
      lines = parse_json_lines(entry.lines)

      # Group lines into debit/credit pairs
      debit_lines = lines.select { |l| (l["debit"]?.try(&.as_f?) || l["debit_amount"]?.try(&.as_f?) || 0.0) > 0 }
      credit_lines = lines.select { |l| (l["credit"]?.try(&.as_f?) || l["credit_amount"]?.try(&.as_f?) || 0.0) > 0 }

      # For each debit line, try to match with credit lines
      debit_lines.each do |debit_line|
        debit_account_id = debit_line["account_id"]?.try(&.as_i64?) || 0_i64
        debit_account_code = debit_line["account_code"]?.try(&.as_s?) || accounts_map[debit_account_id]?.try(&.[:code]) || debit_account_id.to_s
        debit_amount = debit_line["debit"]?.try(&.as_f?) || debit_line["debit_amount"]?.try(&.as_f?) || 0.0
        line_desc = debit_line["description"]?.try(&.as_s?) || entry.description

        # Filter by account if specified
        if account_code && !account_code.empty?
          next unless debit_account_code == account_code
        end

        # Find matching credit line (simplified - take first available)
        credit_line = credit_lines.shift?
        credit_account_code = ""
        credit_amount = 0.0

        if credit_line
          credit_account_id = credit_line["account_id"]?.try(&.as_i64?) || 0_i64
          credit_account_code = credit_line["account_code"]?.try(&.as_s?) || accounts_map[credit_account_id]?.try(&.[:code]) || credit_account_id.to_s
          credit_amount = credit_line["credit"]?.try(&.as_f?) || credit_line["credit_amount"]?.try(&.as_f?) || 0.0
        end

        row_num += 1
        report_rows << {
          row_number:       row_num,
          entry_id:         entry.id,
          entry_date:       entry_date_display,
          document_number:  entry.document_number || "",
          description:      line_desc,
          counterpart_name: counterpart_name,
          debit_account:    debit_account_code,
          credit_account:   credit_account_code,
          debit_amount:     debit_amount,
          credit_amount:    credit_amount,
        }
      end

      # Handle remaining credit lines (if any)
      credit_lines.each do |credit_line|
        credit_account_id = credit_line["account_id"]?.try(&.as_i64?) || 0_i64
        credit_account_code = credit_line["account_code"]?.try(&.as_s?) || accounts_map[credit_account_id]?.try(&.[:code]) || credit_account_id.to_s
        credit_amount = credit_line["credit"]?.try(&.as_f?) || credit_line["credit_amount"]?.try(&.as_f?) || 0.0
        line_desc = credit_line["description"]?.try(&.as_s?) || entry.description

        # Filter by account if specified
        if account_code && !account_code.empty?
          next unless credit_account_code == account_code
        end

        row_num += 1
        report_rows << {
          row_number:       row_num,
          entry_id:         entry.id,
          entry_date:       entry_date_display,
          document_number:  entry.document_number || "",
          description:      line_desc,
          counterpart_name: counterpart_name,
          debit_account:    "",
          credit_account:   credit_account_code,
          debit_amount:     0.0,
          credit_amount:    credit_amount,
        }
      end
    end

    # Sort by date
    report_rows = report_rows.sort_by { |r| r[:entry_date] }

    # Recalculate row numbers after sorting
    report_rows = report_rows.map_with_index do |row, idx|
      row.merge({row_number: idx + 1})
    end

    # Calculate totals
    total_debit = report_rows.sum(&.[:debit_amount])
    total_credit = report_rows.sum(&.[:credit_amount])

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
          account_code:   account_code,
          counterpart_id: target_counterpart_id,
        },
        rows:      report_rows,
        totals:    {
          debit:  total_debit,
          credit: total_credit,
        },
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
