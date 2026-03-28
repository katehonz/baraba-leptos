class Api::Companies::Dashboard < ApiAction
  include Api::Auth::SkipRequireAuthToken

  get "/api/companies/:company_id/dashboard" do
    cid = company_id.to_i64

    # Count journal entries (posted)
    journal_count = JournalEntryQuery.new.company_id(cid).status("posted").select_count.to_i

    # Count counterparts
    counterpart_count = CounterpartQuery.new.company_id(cid).select_count.to_i

    # Calculate revenue (account code 7xx) and expenses (account code 6xx)
    revenue = 0.0
    expenses = 0.0

    entries = JournalEntryQuery.new.company_id(cid).status("posted")
    # Build account code lookup
    accounts = AccountQuery.new.company_id(cid)
    account_codes = {} of Int64 => String
    accounts.each { |a| account_codes[a.id] = a.code }

    entries.each do |entry|
      next if entry.lines.nil? || entry.lines.not_nil!.empty?
      begin
        lines = JSON.parse(entry.lines.not_nil!)
        next unless lines.as_a?
        lines.as_a.each do |line|
          account_id = line["account_id"]?.try(&.as_i64?) || 0_i64
          code = account_codes[account_id]? || ""
          debit = line["debit"]?.try(&.as_f?) || line["debit_amount"]?.try(&.as_f?) || 0.0
          credit = line["credit"]?.try(&.as_f?) || line["credit_amount"]?.try(&.as_f?) || 0.0

          if code.starts_with?("7") # Revenue accounts
            revenue += credit - debit
          elsif code.starts_with?("6") # Expense accounts
            expenses += debit - credit
          end
        end
      rescue
        next
      end
    end

    json({
      success: true,
      data:    {
        journal_entries:  journal_count,
        counterparts:     counterpart_count,
        revenue:          revenue.round(2),
        expenses:         expenses.round(2),
        profit:           (revenue - expenses).round(2),
      },
    })
  end
end
