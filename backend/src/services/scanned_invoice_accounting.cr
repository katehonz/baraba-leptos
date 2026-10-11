# Books scanned invoices and their payments as separate journal entries.
# Invoice JE: base + VAT vs counterpart. Payment JE: counterpart vs cash/card buffer.
class ScannedInvoiceAccounting
  class Error < Exception
  end

  def self.book_invoice!(invoice : ScannedInvoice, company : Company) : ScannedInvoice
    raise Error.new("Фактурата вече е осчетоводена") if invoice.journal_entry_id

    counterpart_account, vat_account, expense_revenue_account = resolve_invoice_accounts(invoice, company)

    if counterpart_account.nil? || expense_revenue_account.nil?
      raise Error.new("Не са конфигурирани сметките за осчетоводяване")
    end

    subtotal, vat_amount, total = amounts(invoice)
    vendor_or_customer = invoice.direction == "purchase" ? invoice.vendor_name : invoice.customer_name
    invoice_num = invoice.invoice_number || "N/A"

    lines = [] of NamedTuple(account_id: Int64, account_code: String, debit: Float64, credit: Float64, description: String)

    if invoice.direction == "purchase"
      lines << line(expense_revenue_account.not_nil!, subtotal, 0.0, "Разход: #{invoice_num}")
      if vat_amount > 0 && vat_account
        lines << line(vat_account.not_nil!, vat_amount, 0.0, "ДДС по фактура #{invoice_num}")
      end
      lines << line(counterpart_account.not_nil!, 0.0, total, "Доставчик: #{vendor_or_customer || "N/A"}")
    else
      lines << line(counterpart_account.not_nil!, total, 0.0, "Клиент: #{vendor_or_customer || "N/A"}")
      lines << line(expense_revenue_account.not_nil!, 0.0, subtotal, "Приход: #{invoice_num}")
      if vat_amount > 0 && vat_account
        lines << line(vat_account.not_nil!, 0.0, vat_amount, "ДДС по фактура #{invoice_num}")
      end
    end

    entry_date = invoice.invoice_date || Time.utc
    period = invoice.vat_period
    vat_period = if period.nil? || period.not_nil!.empty?
                   entry_date.to_s("%Y-%m")
                 else
                   period.not_nil!
                 end
    description = "Фактура #{invoice_num} от #{vendor_or_customer || "N/A"}"

    entry = SaveJournalEntry.create!(
      company_id: invoice.company_id,
      entry_date: entry_date,
      description: description,
      reference: "AI-SCAN-#{invoice.id}",
      document_number: invoice.invoice_number,
      document_date: invoice.invoice_date,
      status: "draft",
      lines: lines_json(lines),
      total_amount: total, # GROSS (основа + ДДС). VAT import uses gross / 1.20 for 20%.
      total_vat_amount: vat_amount,
      counterpart_id: invoice.counterpart_id,
      vat_period: vat_period,
      vat_purchase_operation: invoice.direction == "purchase" ? "01|PURCHASE_CREDIT" : nil,
      vat_sales_operation: invoice.direction == "sale" ? "01|DOMESTIC_SALE" : nil
    )

    SaveScannedInvoice.update!(invoice,
      journal_entry_id: entry.id,
      status: "processed"
    )
  end

  def self.ensure_invoice_booked!(invoice : ScannedInvoice, company : Company) : ScannedInvoice
    return invoice if invoice.journal_entry_id
    book_invoice!(invoice, company)
  end

  def self.book_payment!(invoice : ScannedInvoice, company : Company, method : String) : ScannedInvoice
    method = method.strip.downcase
    unless method == "cash" || method == "card"
      raise Error.new("Невалиден начин на плащане. Използвайте cash или card")
    end

    raise Error.new("Фактурата вече има статия за плащане") if invoice.payment_journal_entry_id

    _subtotal, _vat_amount, total = amounts(invoice)
    if total <= 0
      raise Error.new("Липсва сума за плащане по фактурата")
    end

    counterpart_account = resolve_counterpart_account(invoice, company)
    raise Error.new("Не е настроена сметката на контрагента (клиенти/доставчици)") unless counterpart_account

    payment_account = resolve_payment_account(invoice, company, method)
    unless payment_account
      if method == "cash"
        raise Error.new("Настройте сметка Каса в Настройки → Сметки по подразбиране")
      else
        label = invoice.direction == "purchase" ? "сметка за плащане с карта (покупки)" : "сметка за плащане с карта (продажби)"
        raise Error.new("Настройте #{label} в Настройки → Сметки по подразбиране")
      end
    end

    invoice = ensure_invoice_booked!(invoice, company)

    invoice_num = invoice.invoice_number || "N/A"
    vendor_or_customer = invoice.direction == "purchase" ? invoice.vendor_name : invoice.customer_name
    method_label = method == "cash" ? "в брой" : "с карта"
    description = "Плащане #{method_label} по фактура #{invoice_num}"
    if name = vendor_or_customer
      description = "#{description} / #{name}" unless name.empty?
    end

    lines = [] of NamedTuple(account_id: Int64, account_code: String, debit: Float64, credit: Float64, description: String)

    if invoice.direction == "purchase"
      # Settle supplier: Dt 401 / Ct cash or card buffer
      lines << line(counterpart_account, total, 0.0, "Плащане #{method_label}: #{invoice_num}")
      lines << line(payment_account, 0.0, total, method == "cash" ? "Каса" : "Карта покупки")
    else
      # Settle customer: Dt cash or card buffer / Ct 411
      lines << line(payment_account, total, 0.0, method == "cash" ? "Каса" : "Карта продажби")
      lines << line(counterpart_account, 0.0, total, "Плащане #{method_label}: #{invoice_num}")
    end

    entry_date = invoice.invoice_date || Time.utc
    saft_code = method == "cash" ? "10" : "48"

    entry = SaveJournalEntry.create!(
      company_id: invoice.company_id,
      entry_date: entry_date,
      description: description,
      reference: "AI-PAY-#{invoice.id}",
      document_number: invoice.invoice_number,
      document_date: invoice.invoice_date,
      status: "draft",
      lines: lines_json(lines),
      total_amount: total,
      total_vat_amount: 0.0,
      counterpart_id: invoice.counterpart_id,
      vat_period: invoice.vat_period,
      payment_method_code: saft_code
    )

    SaveScannedInvoice.update!(invoice,
      payment_journal_entry_id: entry.id,
      payment_method: method,
      status: "processed"
    )
  end

  private def self.amounts(invoice : ScannedInvoice)
    subtotal = (invoice.subtotal || 0.0).round(2)
    vat_amount = (invoice.total_tax || 0.0).round(2)
    total = (invoice.invoice_total || (subtotal + vat_amount)).round(2)
    {subtotal, vat_amount, total}
  end

  private def self.resolve_invoice_accounts(invoice : ScannedInvoice, company : Company)
    counterpart_account = find_account(invoice.company_id, invoice.counterparty_account_id)
    vat_account = find_account(invoice.company_id, invoice.vat_account_id)
    expense_revenue_account = find_account(invoice.company_id, invoice.expense_revenue_account_id)

    if counterpart_account.nil?
      account_id = invoice.direction == "purchase" ? company.suppliers_account_id : company.customers_account_id
      counterpart_account = find_account(invoice.company_id, account_id)
    end

    if expense_revenue_account.nil?
      account_id = invoice.direction == "purchase" ? company.expenses_account_id : company.revenues_account_id
      expense_revenue_account = find_account(invoice.company_id, account_id)
    end

    if vat_account.nil?
      account_id = invoice.direction == "purchase" ? company.vat_receivable_account_id : company.vat_payable_account_id
      vat_account = find_account(invoice.company_id, account_id)
    end

    {counterpart_account, vat_account, expense_revenue_account}
  end

  private def self.resolve_counterpart_account(invoice : ScannedInvoice, company : Company) : Account?
    account = find_account(invoice.company_id, invoice.counterparty_account_id)
    return account if account

    account_id = invoice.direction == "purchase" ? company.suppliers_account_id : company.customers_account_id
    find_account(invoice.company_id, account_id)
  end

  private def self.resolve_payment_account(invoice : ScannedInvoice, company : Company, method : String) : Account?
    account_id = if method == "cash"
                   company.cash_account_id
                 elsif invoice.direction == "purchase"
                   company.card_purchase_account_id
                 else
                   company.card_sales_account_id
                 end
    find_account(invoice.company_id, account_id)
  end

  private def self.find_account(company_id : Int64, account_id : Int64?) : Account?
    return nil unless account_id
    AccountQuery.new.company_id(company_id).id(account_id).first?
  end

  private def self.line(account : Account, debit : Float64, credit : Float64, description : String)
    {
      account_id:  account.id,
      account_code: account.code,
      debit:       debit.round(2),
      credit:      credit.round(2),
      description: description,
    }
  end

  private def self.lines_json(lines)
    lines.map do |line|
      {
        "account_id"    => line[:account_id],
        "account_code"  => line[:account_code],
        "debit"         => line[:debit],
        "credit"        => line[:credit],
        "description"   => line[:description],
      }
    end.to_json
  end
end
