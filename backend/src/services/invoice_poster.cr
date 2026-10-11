class InvoicePoster
  class Error < Exception
  end

  def self.issue!(invoice : Invoice, company : Company) : Invoice
    raise Error.new("Не е чернова") unless invoice.draft?
    DocumentNumberService.initialize_for_company(company)
    PeriodService.validate_period_open!(company.id, invoice.issue_date)
    number = invoice.invoice_number
    seq_id = invoice.sequence_id
    if invoice.sale?
      if number.nil? || number.try(&.blank?)
        seq = DocumentNumberService.sequence_for(company.id, "invoice_sale", invoice.document_type)
        raise Error.new("Няма номерационна верига") unless seq
        seq_id = seq.id
        number = DocumentNumberService.allocate!(seq.id)
      elsif seq_id.nil?
        seq = DocumentNumberService.sequence_for(company.id, "invoice_sale", invoice.document_type)
        seq_id = seq.try(&.id)
      end
    else
      raise Error.new("Покупката изисква номер на доставчика") if number.nil? || number.try(&.blank?)
    end
    SaveInvoice.update!(invoice,
      invoice_number: number,
      sequence_id: seq_id,
      status: "issued",
      issued_at: Time.utc
    )
    InvoiceQuery.new.id(invoice.id).first.not_nil!
  end

  def self.post!(invoice : Invoice, company : Company, user : User) : Invoice
    invoice = issue!(invoice, company) if invoice.draft?
    raise Error.new("Вече е осчетоводена") if invoice.posted?
    if invoice.document_type == "PROFORMA"
      raise Error.new("Проформата не е данъчен документ и не се осчетоводява")
    end
    unless Invoice::TAX_DOC_TYPES.includes?(invoice.document_type)
      raise Error.new("Този вид документ не се осчетоводява")
    end
    nap = (invoice.nap_document_type || "").strip
    nap = default_nap(invoice.document_type) if nap.empty?
    allowed = invoice.purchase? ? VatJournalEntry::PURCHASE_DOC_TYPES : VatJournalEntry::SALES_DOC_TYPES
    unless allowed.has_key?(nap)
      raise Error.new("Невалиден вид документ за дневника по ДДС")
    end
    PeriodService.validate_period_open!(company.id, invoice.issue_date)
    if ted = invoice.tax_event_date
      PeriodService.validate_period_open!(company.id, ted)
    end
    forbid_self_deal!(invoice, company)

    lines = InvoiceLineQuery.new.invoice_id(invoice.id)
    raise Error.new("Няма редове") if lines.select_count == 0

    rebuild_json_cache(invoice, lines)

    je = build_journal!(invoice, company, user, nap, lines)
    SaveInvoice.update!(invoice,
      journal_entry_id: je.id,
      status: "posted",
      nap_document_type: nap
    )
    posted = InvoiceQuery.new.id(invoice.id).first.not_nil!

    if lines.any? { |l| WarehouseDocumentPoster.inventory_line?(l) }
      kind = warehouse_kind(posted)
      wd = WarehouseDocumentPoster.ensure_for_invoice!(posted, company, kind)
      allow_neg = company.allow_negative_stock?
      WarehouseDocumentPoster.post!(wd, company, allow_neg)
    end
    posted
  end

  def self.issue_and_post!(invoice : Invoice, company : Company, user : User) : Invoice
    post!(invoice, company, user)
  end

  def self.cancel!(invoice : Invoice, reason : String?) : Invoice
    raise Error.new("Осчетоводена фактура се сторнира с unpost или КИ") if invoice.posted?
    SaveInvoice.update!(invoice,
      status: "cancelled",
      cancelled_at: Time.utc,
      cancellation_reason: reason
    )
    InvoiceQuery.new.id(invoice.id).first.not_nil!
  end

  def self.unpost!(invoice : Invoice, company : Company, user : User) : Invoice
    raise Error.new("Не е осчетоводена") unless invoice.posted?
    PeriodService.validate_period_open!(company.id, invoice.issue_date)
    AppDatabase.transaction do
      release_journal!(invoice, delete_drafts: false)
      if wd = WarehouseDocumentQuery.new.invoice_id(invoice.id).first?
        WarehouseDocumentPoster.unpost!(wd, company, true) if wd.status == "posted"
      end
      SaveInvoice.update!(invoice, status: "issued", journal_entry_id: nil)
    end
    InvoiceQuery.new.id(invoice.id).first.not_nil!
  end

  # Връща документа в чернова, за да може да се редактира цялостно.
  # Счетоводна статия в чернова се изтрива. Осчетоводена статия се сторнира
  # и остава в журнала — не се трие.
  def self.return_to_draft!(invoice : Invoice, company : Company, user : User) : NamedTuple(invoice: Invoice, message: String)
    raise Error.new("Анулиран документ не се връща в чернова") if invoice.status == "cancelled"
    raise Error.new("Документът вече е чернова") if invoice.draft?
    PeriodService.validate_period_open!(company.id, invoice.issue_date)
    if ted = invoice.tax_event_date
      PeriodService.validate_period_open!(company.id, ted)
    end

    note = "Фактурата е върната в чернова."
    AppDatabase.transaction do
      note = release_journal!(invoice, delete_drafts: true)
      if wd = WarehouseDocumentQuery.new.invoice_id(invoice.id).first?
        WarehouseDocumentPoster.unpost!(wd, company, true) if wd.status == "posted"
      end
      SaveInvoice.update!(invoice, status: "draft", journal_entry_id: nil)
    end
    fresh = InvoiceQuery.new.id(invoice.id).first.not_nil!
    {invoice: fresh, message: note}
  end

  # Връща текст за това какво е станало със свързаната статия.
  private def self.release_journal!(invoice : Invoice, delete_drafts : Bool) : String
    je_id = invoice.journal_entry_id
    return "Фактурата е върната в чернова." unless je_id
    je = JournalEntryQuery.new.id(je_id).first?
    return "Фактурата е върната в чернова." unless je

    if je.status == "draft"
      if delete_drafts
        DeleteJournalEntry.delete!(je)
        return "Фактурата е върната в чернова. Черновата счетоводна статия е изтрита."
      end
      return "Фактурата е върната в чернова."
    end

    if je.status == "posted"
      SaveJournalEntry.update!(je,
        status: "reversed",
        vat_purchase_operation: nil,
        vat_sales_operation: nil
      )
      return "Фактурата е върната в чернова. Осчетоводената статия е сторнирана и остава в журнала."
    end

    "Фактурата е върната в чернова."
  end

  def self.rebuild_json_cache(invoice : Invoice, lines) : Nil
    arr = lines.map do |l|
      {
        "product_code" => (l.product_id ? ProductQuery.new.id(l.product_id.not_nil!).first?.try(&.code) : nil) || "",
        "description"  => l.description,
        "quantity"     => l.quantity,
        "unit"         => l.unit_code,
        "unit_price"   => l.unit_price,
        "vat_rate"     => l.vat_rate,
        "subtotal"     => l.line_total,
        "vat_amount"   => l.vat_amount,
        "total"        => l.line_total,
        "tax_code"     => l.tax_code || "100211",
      }
    end
    SaveInvoice.update!(invoice, lines: arr.to_json)
  end

  private def self.default_nap(document_type : String) : String
    case document_type
    when "DEBIT_NOTE"   then "02"
    when "CREDIT_NOTE"  then "03"
    when "PROTOCOL"     then "09"
    when "SALES_REPORT" then "81"
    else                     "01"
    end
  end

  private def self.warehouse_kind(invoice : Invoice) : String
    if invoice.sale?
      invoice.document_type == "CREDIT_NOTE" ? "SALES_RETURN" : "SALES_ISSUE"
    else
      invoice.document_type == "CREDIT_NOTE" ? "PURCHASE_RETURN" : "RECEIPT"
    end
  end

  private def self.forbid_self_deal!(invoice : Invoice, company : Company)
    cp = CounterpartQuery.new.id(invoice.counterpart_id).first
    return unless cp
    eik = (cp.eik || "").gsub(/\s+/, "")
    vat = (cp.vat_number || "").gsub(/\s+/, "").upcase
    ceik = (company.eik || "").gsub(/\s+/, "")
    cvat = (company.vat_number || "").gsub(/\s+/, "").upcase
    if (eik.presence && eik == ceik) || (vat.presence && cvat.presence && vat == cvat)
      raise Error.new("Вътрешен трансфер не се прави като фактура към собствената фирма")
    end
  end

  private def self.build_journal!(invoice, company, user, nap, lines)
    rows = lines.to_a
    subtotal = rows.sum(&.line_total)
    vat = rows.sum(&.vat_amount)
    total = rows.sum(&.line_total_with_vat)
    vat_op = "#{nap}|FROM_INVOICE"
    counterpart_acc, vat_acc = settlement_accounts(company, invoice)
    if invoice.sale?
      raise Error.new("Липсва сметка за клиенти в настройките на фирмата") unless counterpart_acc
    else
      raise Error.new("Липсва сметка за доставчици в настройките на фирмата") unless counterpart_acc
    end

    groups = {} of Int64 => Float64
    rows.each do |line|
      acc = pl_account_id(line, invoice)
      raise Error.new(missing_account_message(line, invoice)) unless acc
      amount = line.line_total.abs
      next if amount <= 0.0001
      groups[acc] = groups.fetch(acc, 0.0) + amount
    end
    raise Error.new("Няма сума за осчетоводяване") if groups.empty?

    abs_vat = vat.abs
    if abs_vat > 0.0001 && vat_acc.nil?
      label = invoice.sale? ? "ДДС продажби" : "ДДС покупки"
      raise Error.new("Липсва сметка за #{label} в настройките на фирмата")
    end

    debit_cp = invoice.sale?
    je_lines = [] of Hash(String, Float64 | Int64 | String)
    abs_total = total.abs

    if debit_cp
      je_lines << line_hash(counterpart_acc.not_nil!, abs_total, 0.0)
      groups.each { |acc, amount| je_lines << line_hash(acc, 0.0, amount) }
      je_lines << line_hash(vat_acc.not_nil!, 0.0, abs_vat) if abs_vat > 0.0001 && !vat_acc.nil?
    else
      groups.each { |acc, amount| je_lines << line_hash(acc, amount, 0.0) }
      je_lines << line_hash(vat_acc.not_nil!, abs_vat, 0.0) if abs_vat > 0.0001 && !vat_acc.nil?
      je_lines << line_hash(counterpart_acc.not_nil!, 0.0, abs_total)
    end

    if invoice.document_type == "CREDIT_NOTE"
      je_lines = je_lines.map do |h|
        d = h["debit"].as(Float64)
        c = h["credit"].as(Float64)
        h.merge({"debit" => c, "credit" => d})
      end
    end

    period_date = invoice.tax_event_date || invoice.issue_date
    vat_period = period_date.to_s("%Y-%m")
    names = invoice.sale? ? VatJournalEntry::SALES_DOC_TYPES : VatJournalEntry::PURCHASE_DOC_TYPES
    label = names[nap]? || invoice.document_type
    SaveJournalEntry.create!(
      company_id: company.id,
      user_id: user.id,
      entry_date: invoice.issue_date,
      description: "#{label} #{invoice.invoice_number}",
      document_number: (invoice.invoice_number || "")[0, 20],
      document_date: invoice.issue_date,
      status: "posted",
      counterpart_id: invoice.counterpart_id,
      vat_period: vat_period,
      vat_sales_operation: invoice.sale? ? vat_op : nil,
      vat_purchase_operation: invoice.purchase? ? vat_op : nil,
      total_amount: subtotal,
      total_vat_amount: vat,
      lines: je_lines.to_json
    )
  end

  private def self.settlement_accounts(company, invoice)
    if invoice.sale?
      {company.customers_account_id, company.vat_payable_account_id}
    else
      {company.suppliers_account_id, company.vat_receivable_account_id}
    end
  end

  # Приход / разход / запаси идват от реда или от артикула.
  # Сметките от настройките на фирмата (приходи/разходи) не се ползват.
  private def self.pl_account_id(line : InvoiceLine, invoice : Invoice) : Int64?
    return line.account_id if line.account_id
    return nil unless line.product_id
    product = ProductQuery.new.id(line.product_id.not_nil!).first?
    return nil unless product
    if invoice.sale?
      product.revenue_account_id
    elsif WarehouseDocumentPoster.inventory_line?(line)
      product.inventory_account_id
    else
      product.expense_account_id
    end
  end

  private def self.missing_account_message(line : InvoiceLine, invoice : Invoice) : String
    kind = if line.product_id.nil? && line.account_id.nil?
             "изберете артикул от номенклатурата или сметка"
           elsif invoice.sale?
             "няма приходна сметка в номенклатурата"
           elsif WarehouseDocumentPoster.inventory_line?(line)
             "няма сметка за материални запаси в номенклатурата"
           else
             "няма разходна сметка в номенклатурата"
           end
    "Ред #{line.line_number} „#{line.description}“: #{kind}"
  end

  private def self.line_hash(account_id : Int64, debit : Float64, credit : Float64)
    acc = AccountQuery.new.id(account_id).first
    {
      "account_id"   => account_id,
      "account_code" => acc.try(&.code) || "",
      "debit"        => debit,
      "credit"       => credit,
    } of String => Float64 | Int64 | String
  end
end
