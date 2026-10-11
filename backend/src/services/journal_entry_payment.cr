# Separate cash/card payment journal entry for a purchase or sale JE.
# Same accounting as AI scanning: counterpart vs каса / card buffer. No VAT on the payment.
class JournalEntryPayment
  class Error < Exception
  end

  def self.book!(entry : JournalEntry, company : Company, method : String) : JournalEntry
    method = method.strip.downcase
    unless method == "cash" || method == "card"
      raise Error.new("Невалиден начин на плащане. Използвайте cash или card")
    end

    if payment_entry?(entry)
      raise Error.new("Това е статия за плащане — не може да се създаде друго плащане от нея")
    end

    if pay_id = existing_payment_id(entry)
      raise Error.new("Вече има статия за плащане ##{pay_id}")
    end

    if scanned = ScannedInvoiceQuery.new.journal_entry_id(entry.id).first?
      updated = ScannedInvoiceAccounting.book_payment!(scanned, company, method)
      if pay_id = updated.payment_journal_entry_id
        return JournalEntryQuery.new.find(pay_id)
      end
      raise Error.new("Плащането не беше създадено")
    end

    direction = direction_for(entry, company)
    counterpart_account = resolve_counterpart_account(entry, company, direction)
    raise Error.new("Не е настроена сметката на контрагента (клиенти/доставчици)") unless counterpart_account

    payment_account = resolve_payment_account(company, method, direction)
    unless payment_account
      if method == "cash"
        raise Error.new("Настройте сметка Каса в Настройки → Сметки по подразбиране")
      else
        label = direction == "purchase" ? "сметка за плащане с карта (покупки)" : "сметка за плащане с карта (продажби)"
        raise Error.new("Настройте #{label} в Настройки → Сметки по подразбиране")
      end
    end

    if includes_account?(entry, payment_account)
      raise Error.new("Този запис вече съдържа каса или карта. Плащането е в същата статия")
    end

    total = payment_amount(entry, counterpart_account, direction)
    if total <= 0
      raise Error.new("Липсва сума за плащане по записа")
    end

    doc_no = entry.document_number
    invoice_num = (doc_no && !doc_no.empty?) ? doc_no : "##{entry.id}"
    method_label = method == "cash" ? "в брой" : "с карта"
    description = "Плащане #{method_label} по документ #{invoice_num}"
    if name = counterpart_name(entry)
      description = "#{description} / #{name}" unless name.empty?
    end

    lines = [] of NamedTuple(account_id: Int64, account_code: String, debit: Float64, credit: Float64, description: String)

    if direction == "purchase"
      lines << line(counterpart_account, total, 0.0, "Плащане #{method_label}: #{invoice_num}")
      lines << line(payment_account, 0.0, total, method == "cash" ? "Каса" : "Карта покупки")
    else
      lines << line(payment_account, total, 0.0, method == "cash" ? "Каса" : "Карта продажби")
      lines << line(counterpart_account, 0.0, total, "Плащане #{method_label}: #{invoice_num}")
    end

    entry_date = entry.document_date || entry.entry_date
    saft_code = method == "cash" ? "10" : "48"

    SaveJournalEntry.create!(
      company_id: entry.company_id,
      entry_date: entry_date,
      description: description,
      reference: "PAY-#{entry.id}",
      document_number: entry.document_number,
      document_date: entry.document_date || entry.entry_date,
      status: "draft",
      lines: lines_json(lines),
      total_amount: total,
      total_vat_amount: 0.0,
      counterpart_id: entry.counterpart_id,
      vat_period: entry.vat_period,
      payment_method_code: saft_code
    )
  end

  def self.existing_payment_id(entry : JournalEntry) : Int64?
    if pay = JournalEntryQuery.new.company_id(entry.company_id).reference("PAY-#{entry.id}").first?
      return pay.id
    end
    if scanned = ScannedInvoiceQuery.new.journal_entry_id(entry.id).first?
      return scanned.payment_journal_entry_id
    end
    nil
  end

  def self.payment_entry?(entry : JournalEntry) : Bool
    ref = entry.reference || ""
    code = entry.payment_method_code || ""
    ref.starts_with?("PAY-") || ref.starts_with?("AI-PAY-") ||
      code == "10" || code == "11" || code == "48"
  end

  def self.direction_for(entry : JournalEntry, company : Company) : String
    if (op = entry.vat_purchase_operation) && !op.empty?
      return "purchase"
    end
    if (op = entry.vat_sales_operation) && !op.empty?
      return "sale"
    end

    if suppliers = find_account(company.id, company.suppliers_account_id)
      return "purchase" if credit_on(entry, suppliers) > 0
    end
    if customers = find_account(company.id, company.customers_account_id)
      return "sale" if debit_on(entry, customers) > 0
    end

    JournalLines.parse(entry.lines).each do |line|
      code = JournalLines.account_code(line) || ""
      return "purchase" if code.starts_with?("401") && JournalLines.credit(line) > 0
      return "sale" if code.starts_with?("411") && JournalLines.debit(line) > 0
    end

    "purchase"
  end

  def self.can_create_payment?(entry : JournalEntry, company : Company) : Bool
    return false if payment_entry?(entry)
    return false if existing_payment_id(entry)
    true
  end

  private def self.resolve_counterpart_account(entry : JournalEntry, company : Company, direction : String) : Account?
    preferred_id = direction == "purchase" ? company.suppliers_account_id : company.customers_account_id
    preferred = find_account(company.id, preferred_id)
    return preferred if preferred && account_used?(entry, preferred)

    prefixes = direction == "purchase" ? ["401"] : ["411"]
    JournalLines.parse(entry.lines).each do |line|
      code = JournalLines.account_code(line) || ""
      next unless prefixes.any? { |p| code.starts_with?(p) }
      if id = JournalLines.account_id(line)
        if acc = find_account(company.id, id)
          return acc
        end
      end
      if acc = AccountQuery.new.company_id(company.id).code(code).first?
        return acc
      end
    end

    preferred
  end

  private def self.resolve_payment_account(company : Company, method : String, direction : String) : Account?
    account_id = if method == "cash"
                   company.cash_account_id
                 elsif direction == "purchase"
                   company.card_purchase_account_id
                 else
                   company.card_sales_account_id
                 end
    find_account(company.id, account_id)
  end

  private def self.payment_amount(entry : JournalEntry, counterpart : Account, direction : String) : Float64
    amt = direction == "purchase" ? credit_on(entry, counterpart) : debit_on(entry, counterpart)
    return amt.round(2) if amt > 0
    JournalLines.document_total(entry.lines, entry.total_amount)
  end

  private def self.credit_on(entry : JournalEntry, account : Account) : Float64
    JournalLines.parse(entry.lines).sum(0.0) do |line|
      JournalLines.matches_account?(line, account.id, account.code) ? JournalLines.credit(line) : 0.0
    end
  end

  private def self.debit_on(entry : JournalEntry, account : Account) : Float64
    JournalLines.parse(entry.lines).sum(0.0) do |line|
      JournalLines.matches_account?(line, account.id, account.code) ? JournalLines.debit(line) : 0.0
    end
  end

  private def self.account_used?(entry : JournalEntry, account : Account) : Bool
    includes_account?(entry, account)
  end

  private def self.includes_account?(entry : JournalEntry, account : Account) : Bool
    JournalLines.parse(entry.lines).any? do |line|
      JournalLines.matches_account?(line, account.id, account.code)
    end
  end

  private def self.find_account(company_id : Int64, account_id : Int64?) : Account?
    return nil unless account_id
    AccountQuery.new.company_id(company_id).id(account_id).first?
  end

  private def self.counterpart_name(entry : JournalEntry) : String?
    return nil unless entry.counterpart_id
    CounterpartQuery.new.id(entry.counterpart_id.not_nil!).first?.try(&.name)
  end

  private def self.line(account : Account, debit : Float64, credit : Float64, description : String)
    {
      account_id:   account.id,
      account_code: account.code,
      debit:        debit.round(2),
      credit:       credit.round(2),
      description:  description,
    }
  end

  private def self.lines_json(lines)
    lines.map do |line|
      {
        "account_id"   => line[:account_id],
        "account_code" => line[:account_code],
        "debit"        => line[:debit],
        "credit"       => line[:credit],
        "description"  => line[:description],
      }
    end.to_json
  end
end
