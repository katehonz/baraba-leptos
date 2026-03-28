class Api::BankTransactions::Book < ApiAction
  post "/api/companies/:company_id/bank_transactions/:bank_transaction_id/book" do
    # Get bank accounts for this company
    account_ids = BankAccountQuery.new.company_id(company_id).map(&.id)

    transaction = BankTransactionQuery.new
      .bank_account_id.in(account_ids)
      .find(bank_transaction_id)

    # Check if already booked
    if transaction.journal_entry_id
      response.status_code = 400
      return json({success: false, error: "Транзакцията вече е осчетоводена"})
    end

    # Get bank account
    bank_account = BankAccountQuery.new.find(transaction.bank_account_id)

    # Get GL account for bank
    gl_account_id = bank_account.gl_account_id
    unless gl_account_id
      response.status_code = 400
      return json({success: false, error: "Банковата сметка няма свързана GL сметка"})
    end

    gl_account = AccountQuery.new.find(gl_account_id)

    # Get contra account from params or use default
    contra_account_id = params.get?("contra_account_id").try(&.to_i64)

    unless contra_account_id
      response.status_code = 400
      return json({success: false, error: "Моля изберете кореспондираща сметка"})
    end

    contra_account = AccountQuery.new.company_id(company_id).find(contra_account_id)

    # Build journal entry lines
    amount = transaction.amount.abs
    is_deposit = transaction.amount > 0

    lines = [] of NamedTuple(account_id: Int64, debit: Float64, credit: Float64, description: String)

    if is_deposit
      # Deposit: Debit Bank, Credit Contra
      lines << {
        account_id:  gl_account.id,
        debit:       amount,
        credit:      0.0,
        description: transaction.description || "Банков депозит",
      }
      lines << {
        account_id:  contra_account.id,
        debit:       0.0,
        credit:      amount,
        description: transaction.contra_name || "Контрагент",
      }
    else
      # Withdrawal: Debit Contra, Credit Bank
      lines << {
        account_id:  contra_account.id,
        debit:       amount,
        credit:      0.0,
        description: transaction.contra_name || "Контрагент",
      }
      lines << {
        account_id:  gl_account.id,
        debit:       0.0,
        credit:      amount,
        description: transaction.description || "Банков превод",
      }
    end

    lines_json = lines.map do |line|
      {
        "account_id"  => line[:account_id],
        "debit"       => line[:debit],
        "credit"      => line[:credit],
        "description" => line[:description],
      }
    end

    description = transaction.description || (is_deposit ? "Банков депозит" : "Банков превод")
    if transaction.contra_name
      description = "#{description} - #{transaction.contra_name}"
    end

    # Create journal entry
    SaveJournalEntry.create(
      company_id: company_id.to_i64,
      entry_date: transaction.date,
      description: description,
      document_number: transaction.reference,
      status: "draft",
      lines: lines_json.to_json,
      total_amount: amount
    ) do |operation, entry|
      if entry
        # Update transaction with journal entry ID
        SaveBankTransaction.update!(transaction, journal_entry_id: entry.id)

        json({
          success: true,
          data:    {
            journal_entry_id: entry.id,
            message:          "Създаден е счетоводен запис ##{entry.id}",
          },
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
