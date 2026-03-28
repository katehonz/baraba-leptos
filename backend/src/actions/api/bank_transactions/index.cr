class Api::BankTransactions::Index < ApiAction
  get "/api/companies/:company_id/bank_transactions" do
    # Get bank accounts for this company
    bank_accounts_map = Hash(Int64, BankAccount).new
    BankAccountQuery.new.company_id(company_id).each do |ba|
      bank_accounts_map[ba.id] = ba
    end
    account_ids = bank_accounts_map.keys

    transactions = BankTransactionQuery.new
      .bank_account_id.in(account_ids)
      .date.desc_order

    # Filter by bank account
    if account_id = params.get?("bank_account_id")
      transactions = transactions.bank_account_id(account_id.to_i64)
    end

    # Filter by date range
    if date_from = params.get?("date_from")
      from_time = Time.parse(date_from.to_s, "%Y-%m-%d", Time::Location::UTC)
      transactions = transactions.date.gte(from_time)
    end
    if date_to = params.get?("date_to")
      to_time = Time.parse(date_to.to_s, "%Y-%m-%d", Time::Location::UTC) + 1.day
      transactions = transactions.date.lt(to_time)
    end

    # Filter by booked status
    if status = params.get?("status")
      case status.to_s
      when "booked"
        transactions = transactions.journal_entry_id.is_not_nil
      when "unbooked"
        transactions = transactions.journal_entry_id.is_nil
      end
    end

    # Limit
    limit = (params.get?("limit").try(&.to_i) || 500).clamp(1, 1000)
    transactions = transactions.limit(limit)

    # Build response with allocation status
    json({
      success: true,
      data:    transactions.map { |tx|
        bank_account = bank_accounts_map[tx.bank_account_id]
        buffer_account_id = bank_account.buffer_account_id

        # Determine allocation status
        is_booked = !tx.journal_entry_id.nil?
        is_allocated = false
        journal_lines_data = nil

        if is_booked && tx.journal_entry_id
          entry = JournalEntryQuery.new.find(tx.journal_entry_id.not_nil!)
          lines_json = entry.lines
          if !lines_json.empty? && lines_json != "[]"
            parsed_lines = JSON.parse(lines_json)
            journal_lines_data = parsed_lines

            # Transaction is allocated if no line uses the buffer account
            if buffer_account_id
              has_buffer = parsed_lines.as_a.any? do |line|
                line_acct = line["account_id"]?.try(&.as_i64?)
                line_acct == buffer_account_id
              end
              is_allocated = !has_buffer
            else
              # No buffer account configured — consider it allocated if booked
              is_allocated = true
            end
          end
        end

        {
          id:                tx.id,
          bank_account_id:   tx.bank_account_id,
          bank_account_name: bank_account.name,
          date:              tx.date,
          amount:            tx.amount,
          currency:          tx.currency,
          amount_base:       tx.amount_base,
          description:       tx.description,
          contra_account:    tx.contra_account,
          contra_name:       tx.contra_name,
          reference:         tx.reference,
          journal_entry_id:  tx.journal_entry_id,
          is_booked:         is_booked,
          is_allocated:      is_allocated,
          journal_lines:     journal_lines_data,
        }
      },
    })
  end
end
