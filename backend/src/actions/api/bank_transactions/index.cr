class Api::BankTransactions::Index < ApiAction
  get "/api/companies/:company_id/bank_transactions" do
    bank_accounts_map = Hash(Int64, BankAccount).new
    BankAccountQuery.new.company_id(company_id).each do |ba|
      bank_accounts_map[ba.id] = ba
    end
    account_ids = bank_accounts_map.keys

    page = (params.get?("page") || "1").to_i
    page = 1 if page < 1
    per_page = (params.get?("per_page") || params.get?("limit") || "25").to_i
    per_page = per_page.clamp(1, 100)

    if account_ids.empty?
      return json({
        success:    true,
        data:       [] of Nil,
        pagination: pagination_json(page, per_page, 0, 0),
      })
    end

    transactions = BankTransactionQuery.new
      .bank_account_id.in(account_ids)
      .date.desc_order
      .id.desc_order

    if account_id = params.get?("bank_account_id")
      unless account_id.to_s.strip.empty?
        transactions = transactions.bank_account_id(account_id.to_i64)
      end
    end

    if date_from = parse_date_param("date_from")
      transactions = transactions.date.gte(date_from)
    end
    if date_to = parse_date_param("date_to")
      transactions = transactions.date.lt(date_to + 1.day)
    end

    if status = params.get?("status")
      unless status.to_s.strip.empty?
        transactions = apply_status_filter(transactions, status.to_s)
      end
    end

    total = transactions.clone.select_count
    total_pages = per_page > 0 ? (total / per_page.to_f).ceil.to_i : 0
    if total_pages > 0 && page > total_pages
      page = total_pages
    end
    offset = (page - 1) * per_page
    transactions = transactions.limit(per_page).offset(offset)

    tx_rows = [] of BankTransaction
    transactions.each { |tx| tx_rows << tx }

    je_ids = tx_rows.compact_map(&.journal_entry_id).uniq
    entries_map = Hash(Int64, JournalEntry).new
    unless je_ids.empty?
      JournalEntryQuery.new.id.in(je_ids).each do |entry|
        entries_map[entry.id] = entry
      end
    end

    json({
      success:    true,
      data:       tx_rows.map { |tx| tx_json(tx, bank_accounts_map[tx.bank_account_id], entries_map) },
      pagination: pagination_json(page, per_page, total, total_pages),
    })
  end

  private def parse_date_param(name : String) : Time?
    raw = params.get?(name).try(&.to_s).try(&.strip)
    return nil if raw.nil? || raw.empty?
    Time.parse(raw, "%Y-%m-%d", Time::Location::UTC)
  rescue
    nil
  end

  private def apply_status_filter(query : BankTransactionQuery, status : String) : BankTransactionQuery
    case status
    when "booked"
      query.journal_entry_id.is_not_nil
    when "unbooked"
      query.journal_entry_id.is_nil
    when "posted"
      query.where(POSTED_SQL)
    when "unallocated"
      query.where("journal_entry_id IS NOT NULL AND (#{EMPTY_OR_MISSING_LINES_SQL} OR #{HAS_BUFFER_LINE_SQL})")
    when "allocated"
      query.where("journal_entry_id IS NOT NULL AND NOT (#{EMPTY_OR_MISSING_LINES_SQL} OR #{HAS_BUFFER_LINE_SQL}) AND NOT (#{POSTED_SQL})")
    else
      query
    end
  end

  private def pagination_json(page : Int32, per_page : Int32, total : Int64 | Int32, total_pages : Int32)
    {
      page:        page,
      per_page:    per_page,
      total:       total,
      total_pages: total_pages,
    }
  end

  private def tx_json(tx : BankTransaction, bank_account : BankAccount, entries_map : Hash(Int64, JournalEntry))
    buffer_account_id = bank_account.buffer_account_id
    is_booked = false
    is_allocated = false
    journal_lines_data = nil
    journal_entry_status = ""

    if je_id = tx.journal_entry_id
      if entry = entries_map[je_id]?
        is_booked = true
        journal_entry_status = entry.status
        lines_json = entry.lines
        if !lines_json.empty? && lines_json != "[]"
          parsed_lines = JSON.parse(lines_json)
          journal_lines_data = parsed_lines

          if buffer_account_id
            has_buffer = parsed_lines.as_a.any? do |line|
              line_acct = line["account_id"]?.try(&.as_i64?)
              line_acct == buffer_account_id
            end
            is_allocated = !has_buffer
          else
            is_allocated = true
          end
        end
      end
    end

    {
      id:                   tx.id,
      bank_account_id:      tx.bank_account_id,
      bank_account_name:    bank_account.name,
      date:                 tx.date,
      amount:               tx.amount,
      currency:             tx.currency,
      amount_base:          tx.amount_base,
      description:          tx.description,
      contra_account:       tx.contra_account,
      contra_name:          tx.contra_name,
      reference:            tx.reference,
      journal_entry_id:     tx.journal_entry_id,
      journal_entry_status: journal_entry_status,
      is_booked:            is_booked,
      is_allocated:         is_allocated,
      journal_lines:        journal_lines_data,
    }
  end

  HAS_BUFFER_LINE_SQL = <<-SQL
    EXISTS (
      SELECT 1
      FROM journal_entries je
      INNER JOIN bank_accounts ba ON ba.id = bank_transactions.bank_account_id
      WHERE je.id = bank_transactions.journal_entry_id
        AND ba.buffer_account_id IS NOT NULL
        AND je.lines IS NOT NULL
        AND btrim(je.lines) <> ''
        AND je.lines <> '[]'
        AND EXISTS (
          SELECT 1
          FROM jsonb_array_elements(je.lines::jsonb) AS line
          WHERE (line->>'account_id')::bigint = ba.buffer_account_id
        )
    )
  SQL

  EMPTY_OR_MISSING_LINES_SQL = <<-SQL
    NOT EXISTS (
      SELECT 1
      FROM journal_entries je
      WHERE je.id = bank_transactions.journal_entry_id
        AND je.lines IS NOT NULL
        AND btrim(je.lines) <> ''
        AND je.lines <> '[]'
    )
  SQL

  POSTED_SQL = <<-SQL
    EXISTS (
      SELECT 1
      FROM journal_entries je
      WHERE je.id = bank_transactions.journal_entry_id
        AND je.status = 'posted'
    )
  SQL
end
