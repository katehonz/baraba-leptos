require "../../../services/bank_file_parser"

class Api::BankTransactions::Preview < ApiAction
  post "/api/companies/:company_id/bank_transactions/preview" do
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

      # Try to find matching bank account by IBAN
      bank_account : BankAccount? = nil
      if result.account_iban
        bank_account = BankAccountQuery.new.company_id(company_id).iban(result.account_iban.not_nil!).first?
      end

      # Check for duplicates
      existing_refs = Set(String).new
      if bank_account
        BankTransactionQuery.new.bank_account_id(bank_account.id).each do |tx|
          if ref = tx.reference
            existing_refs << ref
          end
        end
      end

      # Prepare preview data
      preview_transactions = result.transactions.first(50).map do |tx|
        is_duplicate = tx.reference && existing_refs.includes?(tx.reference.not_nil!)

        {
          date:          tx.date,
          amount:        tx.amount,
          currency:      tx.currency,
          description:   tx.description,
          contra_account: tx.contra_account,
          contra_name:   tx.contra_name,
          reference:     tx.reference,
          is_duplicate:  is_duplicate,
        }
      end

      duplicate_count = result.transactions.count { |tx| tx.reference && existing_refs.includes?(tx.reference.not_nil!) }

      json({
        success: true,
        data:    {
          bank_format:      result.bank_format,
          account_iban:     result.account_iban,
          account_currency: result.account_currency,
          period_from:      result.period_from,
          period_to:        result.period_to,
          opening_balance:  result.opening_balance,
          closing_balance:  result.closing_balance,
          total_count:      result.transactions.size,
          duplicate_count:  duplicate_count,
          new_count:        result.transactions.size - duplicate_count,
          bank_account_id:  bank_account.try(&.id),
          bank_account_name: bank_account.try(&.name),
          transactions:     preview_transactions,
        },
      })
    rescue ex
      response.status_code = 400
      json({success: false, error: "Грешка при парсване: #{ex.message}"})
    end
  end
end
