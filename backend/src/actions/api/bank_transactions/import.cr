require "../../../services/bank_file_parser"

class Api::BankTransactions::Import < ApiAction
  post "/api/companies/:company_id/bank_transactions/import" do
    bank_account_id = params.get?("bank_account_id").try(&.to_i64)

    # Get file content (base64 encoded)
    file_content_b64 = params.get?("file_content").try(&.to_s)
    if file_content_b64.nil? || file_content_b64.empty?
      response.status_code = 400
      return json({success: false, error: "Не е предоставен файл"})
    end

    # Decode base64
    file_content = Base64.decode_string(file_content_b64)

    begin
      # Parse the file
      result = BankFileParser.parse(file_content)

      # Auto-detect or use provided bank account
      bank_account : BankAccount? = nil

      if bank_account_id
        bank_account = BankAccountQuery.new.company_id(company_id).id(bank_account_id).first?
      elsif result.account_iban
        # Try to find by IBAN
        bank_account = BankAccountQuery.new.company_id(company_id).iban(result.account_iban.not_nil!).first?
      end

      # Count duplicates and new
      existing_refs = Set(String).new
      if bank_account
        BankTransactionQuery.new.bank_account_id(bank_account.id).each do |tx|
          if ref = tx.reference
            existing_refs << ref
          end
        end
      end

      # Resolve GL and buffer accounts for auto-journaling
      gl_account_id = bank_account.try(&.gl_account_id)
      buffer_account_id = bank_account.try(&.buffer_account_id)
      auto_journal = !!(gl_account_id && buffer_account_id)

      new_count = 0
      duplicate_count = 0
      imported_count = 0
      journal_count = 0

      result.transactions.each do |tx|
        # Check for duplicate by reference
        if tx.reference && existing_refs.includes?(tx.reference.not_nil!)
          duplicate_count += 1
          next
        end

        new_count += 1

        # Import if bank account is known
        if bank_account
          bank_tx = SaveBankTransaction.create!(
            bank_account_id: bank_account.id,
            date: tx.date,
            amount: tx.amount,
            currency: tx.currency,
            description: tx.description,
            contra_account: tx.contra_account,
            contra_name: tx.contra_name,
            reference: tx.reference
          )
          imported_count += 1
          existing_refs << tx.reference.not_nil! if tx.reference

          # Auto-create journal entry if GL + buffer accounts are configured
          if auto_journal
            amount = tx.amount.abs
            is_deposit = tx.amount > 0

            lines = [] of Hash(String, Int64 | Float64 | String)

            if is_deposit
              # Deposit: Debit Bank, Credit Buffer
              lines << {
                "account_id"  => gl_account_id.not_nil!.as(Int64 | Float64 | String),
                "debit"       => amount.as(Int64 | Float64 | String),
                "credit"      => 0.0.as(Int64 | Float64 | String),
                "description" => (tx.description || "Банков депозит").as(Int64 | Float64 | String),
              }
              lines << {
                "account_id"  => buffer_account_id.not_nil!.as(Int64 | Float64 | String),
                "debit"       => 0.0.as(Int64 | Float64 | String),
                "credit"      => amount.as(Int64 | Float64 | String),
                "description" => (tx.contra_name || "Контрагент").as(Int64 | Float64 | String),
              }
            else
              # Withdrawal: Debit Buffer, Credit Bank
              lines << {
                "account_id"  => buffer_account_id.not_nil!.as(Int64 | Float64 | String),
                "debit"       => amount.as(Int64 | Float64 | String),
                "credit"      => 0.0.as(Int64 | Float64 | String),
                "description" => (tx.contra_name || "Контрагент").as(Int64 | Float64 | String),
              }
              lines << {
                "account_id"  => gl_account_id.not_nil!.as(Int64 | Float64 | String),
                "debit"       => 0.0.as(Int64 | Float64 | String),
                "credit"      => amount.as(Int64 | Float64 | String),
                "description" => (tx.description || "Банков превод").as(Int64 | Float64 | String),
              }
            end

            description = "Банково извлечение #{bank_account.not_nil!.name} — #{tx.description || ""}"
            if tx.contra_name
              description = "#{description} - #{tx.contra_name}"
            end

            SaveJournalEntry.create(
              company_id: company_id.to_i64,
              entry_date: tx.date,
              description: description,
              document_number: tx.reference,
              status: "draft",
              lines: lines.to_json,
              total_amount: amount
            ) do |operation, entry|
              if entry
                SaveBankTransaction.update!(bank_tx, journal_entry_id: entry.id)
                journal_count += 1
              end
            end
          end
        end
      end

      json({
        success: true,
        data:    {
          bank_format:       result.bank_format,
          account_iban:      result.account_iban,
          account_currency:  result.account_currency,
          period_from:       result.period_from,
          period_to:         result.period_to,
          opening_balance:   result.opening_balance,
          closing_balance:   result.closing_balance,
          total_count:       result.transactions.size,
          new_count:         new_count,
          duplicate_count:   duplicate_count,
          imported_count:    imported_count,
          journal_count:     journal_count,
          auto_journal:      auto_journal,
          bank_account_id:   bank_account.try(&.id),
          bank_account_name: bank_account.try(&.name),
        },
      })
    rescue ex
      response.status_code = 400
      json({success: false, error: "Грешка при парсване: #{ex.message}"})
    end
  end
end
