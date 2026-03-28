class Api::JournalEntries::Show < ApiAction
  get "/api/companies/:company_id/journal_entries/:entry_id" do
    entry = JournalEntryQuery.new.company_id(company_id).id(entry_id).first

    # First try to get lines from journal_lines table
    db_lines = JournalLineQuery.new.journal_entry_id(entry.id)

    # If no lines in table, try to parse from JSON column
    if db_lines.none?
      json_lines = parse_json_lines(entry.lines)
      total_debit = json_lines.sum { |l| l["debit"]?.try(&.as_f?) || l["debit_amount"]?.try(&.as_f?) || 0.0 }
      total_credit = json_lines.sum { |l| l["credit"]?.try(&.as_f?) || l["credit_amount"]?.try(&.as_f?) || 0.0 }

      json({
        success: true,
        data:    {
          id:                     entry.id,
          entry_date:             entry.entry_date,
          document_date:          entry.document_date.try(&.to_s("%Y-%m-%d")),
          description:            entry.description,
          reference:              entry.reference,
          status:                 entry.status,
          document_number:        entry.document_number,
          vat_period:             entry.vat_period,
          vat_purchase_operation: entry.vat_purchase_operation,
          vat_sales_operation:    entry.vat_sales_operation,
          counterpart_id:         entry.counterpart_id,
          lines:                  json_lines.map { |l|
            {
              id:            l["id"]?.try(&.as_i64?) || 0_i64,
              account_id:    l["account_id"]?.try(&.as_i64?) || 0_i64,
              account_code:  l["account_code"]?.try(&.as_s?) || "",
              debit_amount:  l["debit"]?.try(&.as_f?) || l["debit_amount"]?.try(&.as_f?) || 0.0,
              credit_amount: l["credit"]?.try(&.as_f?) || l["credit_amount"]?.try(&.as_f?) || 0.0,
              description:   l["description"]?.try(&.as_s?) || "",
            }
          },
          total_debit:  total_debit,
          total_credit: total_credit,
        },
      })
    else
      json({
        success: true,
        data:    {
          id:                     entry.id,
          entry_date:             entry.entry_date,
          document_date:          entry.document_date.try(&.to_s("%Y-%m-%d")),
          description:            entry.description,
          reference:              entry.reference,
          status:                 entry.status,
          document_number:        entry.document_number,
          vat_period:             entry.vat_period,
          vat_purchase_operation: entry.vat_purchase_operation,
          vat_sales_operation:    entry.vat_sales_operation,
          counterpart_id:         entry.counterpart_id,
          lines:                  db_lines.map { |l|
            {
              id:            l.id,
              account_id:    l.account_id,
              debit_amount:  l.debit_amount,
              credit_amount: l.credit_amount,
              description:   l.description,
            }
          },
          total_debit:  db_lines.sum(&.debit_amount),
          total_credit: db_lines.sum(&.credit_amount),
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
