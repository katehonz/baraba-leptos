class Api::Reports::Chronological < ApiAction
  get "/api/companies/:company_id/reports/chronological" do
    date_from = params.get?("date_from")
    date_to = params.get?("date_to")
    account_code = params.get?("account_code")
    counterpart_id_param = params.get?("counterpart_id")
    target_counterpart_id = counterpart_id_param.try(&.to_i64?)

    company = CompanyQuery.new.id(company_id).first

    id_to_code = {} of Int64 => String
    AccountQuery.new.company_id(company_id).each do |acc|
      id_to_code[acc.id] = acc.code
    end

    counterparts_map = {} of Int64 => String
    CounterpartQuery.new.company_id(company_id).each do |cp|
      counterparts_map[cp.id] = cp.name
    end

    entries_query = JournalEntryQuery.new.company_id(company_id).status("posted")
    if target_counterpart_id
      entries_query = entries_query.counterpart_id(target_counterpart_id)
    end
    entries = entries_query.to_a

    # Chronological register is ordered by the document date of the journal
    # entry (fallback: accounting date). Filter the selected period the same way.
    entries = entries.select do |entry|
      doc_iso = document_date_iso(entry)
      next false if date_from && !date_from.empty? && doc_iso < date_from
      next false if date_to && !date_to.empty? && doc_iso > date_to
      true
    end

    entries = entries.sort_by do |entry|
      {
        document_date_iso(entry),
        entry.entry_date.to_s("%Y-%m-%d"),
        entry.document_number || "",
        entry.id,
      }
    end

    report_rows = [] of NamedTuple(
      row_number: Int32,
      entry_id: Int64,
      entry_date: String,
      document_date: String,
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
      entry_date_display = entry.entry_date.to_s("%d.%m.%Y")
      doc_date_display = format_document_date(entry)

      counterpart_name = ""
      if cp_id = entry.counterpart_id
        counterpart_name = counterparts_map[cp_id]? || ""
      end

      document_number = entry.document_number
      if document_number.nil? || document_number.empty?
        document_number = entry.reference
      end
      document_number ||= ""

      lines = JournalLines.parse(entry.lines)
      debit_lines = lines.select { |l| JournalLines.debit(l) > 0 }
      credit_lines = lines.select { |l| JournalLines.credit(l) > 0 }

      debit_lines.each do |debit_line|
        debit_account_code = JournalLines.resolve_account_code(debit_line, id_to_code)
        debit_amount = JournalLines.debit(debit_line)
        line_desc = JournalLines.description(debit_line) || entry.description

        credit_line = credit_lines.shift?
        credit_account_code = ""
        credit_amount = 0.0

        if credit_line
          credit_account_code = JournalLines.resolve_account_code(credit_line, id_to_code)
          credit_amount = JournalLines.credit(credit_line)
        end

        next unless account_filter_matches?(account_code, debit_account_code, credit_account_code)

        row_num += 1
        report_rows << {
          row_number:       row_num,
          entry_id:         entry.id,
          entry_date:       entry_date_display,
          document_date:    doc_date_display,
          document_number:  document_number,
          description:      line_desc,
          counterpart_name: counterpart_name,
          debit_account:    debit_account_code,
          credit_account:   credit_account_code,
          debit_amount:     debit_amount,
          credit_amount:    credit_amount,
        }
      end

      credit_lines.each do |credit_line|
        credit_account_code = JournalLines.resolve_account_code(credit_line, id_to_code)
        credit_amount = JournalLines.credit(credit_line)
        line_desc = JournalLines.description(credit_line) || entry.description

        next unless account_filter_matches?(account_code, "", credit_account_code)

        row_num += 1
        report_rows << {
          row_number:       row_num,
          entry_id:         entry.id,
          entry_date:       entry_date_display,
          document_date:    doc_date_display,
          document_number:  document_number,
          description:      line_desc,
          counterpart_name: counterpart_name,
          debit_account:    "",
          credit_account:   credit_account_code,
          debit_amount:     0.0,
          credit_amount:    credit_amount,
        }
      end
    end

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

  private def document_date_iso(entry : JournalEntry) : String
    (entry.document_date || entry.entry_date).to_s("%Y-%m-%d")
  end

  private def format_document_date(entry : JournalEntry) : String
    (entry.document_date || entry.entry_date).to_s("%d.%m.%Y")
  end

  # Pair first, then keep the row if either side matches. Filtering debit
  # lines before pairing left invoice 401 credits with an empty debit account
  # and dropped bank payments that stored only account_id.
  private def account_filter_matches?(filter : String?, debit_code : String, credit_code : String) : Bool
    return true if filter.nil? || filter.empty?
    AccountPattern.matches?(debit_code, filter) || AccountPattern.matches?(credit_code, filter)
  end
end
