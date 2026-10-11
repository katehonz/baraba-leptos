class BankTransactionAllocator
  class Error < Exception
  end

  def self.reallocate!(
    company_id : Int64,
    transaction : BankTransaction,
    new_account_id : Int64,
    counterpart_id : Int64? = nil,
  ) : JournalEntry
    unless transaction.journal_entry_id
      raise Error.new("Транзакцията няма счетоводен запис")
    end

    new_account = AccountQuery.new.company_id(company_id).find(new_account_id)
    bank_account = BankAccountQuery.new.find(transaction.bank_account_id)
    buffer_account_id = bank_account.buffer_account_id
    unless buffer_account_id
      raise Error.new("Банковата сметка няма буферна сметка")
    end

    entry = JournalEntryQuery.new.find(transaction.journal_entry_id.not_nil!)
    lines_json = entry.lines
    if lines_json.empty? || lines_json == "[]"
      raise Error.new("Счетоводният запис няма редове")
    end

    parsed_lines = JournalLines.parse(lines_json)
    updated = false
    new_lines = parsed_lines.map do |line|
      hash = (line.as_h? || {} of String => JSON::Any).dup
      line_acct = JournalLines.account_id(line)
      if line_acct == buffer_account_id
        updated = true
        hash["account_id"] = JSON::Any.new(new_account_id)
        hash["account_code"] = JSON::Any.new(new_account.code)
        if cid = counterpart_id
          if JournalLines.settlement_account?(new_account.code)
            hash["counterpart_id"] = JSON::Any.new(cid)
          end
        end
        if JournalLines.description(line).nil?
          hint = transaction.contra_name || transaction.description || "Контрагент"
          hash["description"] = JSON::Any.new(hint)
        end
        JSON::Any.new(hash)
      else
        JSON::Any.new(hash)
      end
    end

    unless updated
      raise Error.new("Не е намерен ред с буферна сметка")
    end

    lines_out = JournalLines.enrich_account_codes(new_lines.to_json, company_id)
    doc_date = entry.document_date || transaction.date
    doc_num = entry.document_number
    if doc_num.nil? || doc_num.empty?
      doc_num = transaction.reference
    end

    if counterpart_id
      SaveJournalEntry.update!(
        entry,
        lines: lines_out,
        counterpart_id: counterpart_id,
        document_date: doc_date,
        document_number: doc_num
      )
    else
      SaveJournalEntry.update!(
        entry,
        lines: lines_out,
        document_date: doc_date,
        document_number: doc_num
      )
    end

    JournalEntryQuery.new.find(entry.id)
  end

  def self.book!(
    company_id : Int64,
    transaction : BankTransaction,
    contra_account_id : Int64,
    counterpart_id : Int64? = nil,
  ) : JournalEntry
    if transaction.journal_entry_id
      raise Error.new("Транзакцията вече е осчетоводена")
    end

    bank_account = BankAccountQuery.new.find(transaction.bank_account_id)
    gl_account_id = bank_account.gl_account_id
    unless gl_account_id
      raise Error.new("Банковата сметка няма свързана GL сметка")
    end

    gl_account = AccountQuery.new.find(gl_account_id)
    contra_account = AccountQuery.new.company_id(company_id).find(contra_account_id)

    amount = transaction.amount.abs
    is_deposit = transaction.amount > 0
    bank_desc = transaction.description || (is_deposit ? "Банков депозит" : "Банков превод")
    contra_desc = transaction.contra_name || "Контрагент"

    lines = [] of Hash(String, Int64 | Float64 | String)
    if is_deposit
      lines << build_line(gl_account, amount, 0.0, bank_desc, nil)
      lines << build_line(contra_account, 0.0, amount, contra_desc, counterpart_id)
    else
      lines << build_line(contra_account, amount, 0.0, contra_desc, counterpart_id)
      lines << build_line(gl_account, 0.0, amount, bank_desc, nil)
    end

    description = bank_desc
    if transaction.contra_name
      description = "#{description} - #{transaction.contra_name}"
    end

    entry = SaveJournalEntry.create!(
      company_id: company_id,
      entry_date: transaction.date,
      document_date: transaction.date,
      description: description,
      document_number: transaction.reference,
      status: "draft",
      lines: lines.to_json,
      total_amount: amount,
      counterpart_id: counterpart_id
    )
    SaveBankTransaction.update!(transaction, journal_entry_id: entry.id)
    entry
  end

  def self.build_line(
    account : Account,
    debit : Float64,
    credit : Float64,
    description : String,
    counterpart_id : Int64?,
  ) : Hash(String, Int64 | Float64 | String)
    line = {
      "account_id"   => account.id,
      "account_code" => account.code,
      "debit"        => debit,
      "credit"       => credit,
      "description"  => description,
    } of String => Int64 | Float64 | String
    if cid = counterpart_id
      if JournalLines.settlement_account?(account.code)
        line["counterpart_id"] = cid
      end
    end
    line
  end
end
