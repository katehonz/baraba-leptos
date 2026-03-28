class Api::BankTransactions::Reallocate < ApiAction
  post "/api/companies/:company_id/bank_transactions/:bank_transaction_id/reallocate" do
    # Get bank accounts for this company
    account_ids = BankAccountQuery.new.company_id(company_id).map(&.id)

    transaction = BankTransactionQuery.new
      .bank_account_id.in(account_ids)
      .find(bank_transaction_id)

    # Must have a journal entry
    unless transaction.journal_entry_id
      response.status_code = 400
      return json({success: false, error: "Транзакцията няма счетоводен запис"})
    end

    # Get the new account to replace buffer with
    new_account_id = params.get?("account_id").try(&.to_i64)
    unless new_account_id
      response.status_code = 400
      return json({success: false, error: "Моля изберете сметка"})
    end

    # Verify the new account exists and belongs to this company
    new_account = AccountQuery.new.company_id(company_id).find(new_account_id)

    # Get bank account and its buffer account
    bank_account = BankAccountQuery.new.find(transaction.bank_account_id)
    buffer_account_id = bank_account.buffer_account_id

    unless buffer_account_id
      response.status_code = 400
      return json({success: false, error: "Банковата сметка няма буферна сметка"})
    end

    # Get the journal entry
    entry = JournalEntryQuery.new.find(transaction.journal_entry_id.not_nil!)

    # Parse and update the lines JSON
    lines_json = entry.lines
    if lines_json.empty? || lines_json == "[]"
      response.status_code = 400
      return json({success: false, error: "Счетоводният запис няма редове"})
    end

    parsed_lines = JSON.parse(lines_json).as_a

    # Find and replace the buffer account line
    updated = false
    new_lines = parsed_lines.map do |line|
      line_acct = line["account_id"]?.try(&.as_i64?)
      if line_acct == buffer_account_id
        updated = true
        # Build new line with replaced account_id
        new_line = Hash(String, JSON::Any).new
        line.as_h.each do |k, v|
          if k == "account_id"
            new_line[k] = JSON::Any.new(new_account_id)
          else
            new_line[k] = v
          end
        end
        JSON::Any.new(new_line)
      else
        line
      end
    end

    unless updated
      response.status_code = 400
      return json({success: false, error: "Не е намерен ред с буферна сметка"})
    end

    # Save updated lines
    SaveJournalEntry.update!(entry, lines: new_lines.to_json)

    json({
      success: true,
      data:    {
        journal_entry_id: entry.id,
        new_account_id:   new_account.id,
        new_account_code: new_account.code,
        new_account_name: new_account.name,
        message:          "Сметката е преразпределена от буфер към #{new_account.code} #{new_account.name}",
      },
    })
  end
end
