class DocumentNumberService
  class LockTimeout < Exception
  end

  class Overflow < Exception
  end

  WH_TYPES = [
    "RECEIPT", "PROD_RECEIPT", "SALES_ISSUE", "SALES_RETURN", "PURCHASE_RETURN",
    "ISSUE", "PROD_ISSUE", "TRANSFER", "INV_PLUS", "INV_MINUS", "WRITE_OFF", "EXPIRY", "OTHER",
  ]

  PADDING    = 10
  MAX_NUMBER = 9_999_999_999_i64

  # Invoice kinds that can share a numbering interval.
  # Proforma is not a tax document and starts in its own interval.
  INVOICE_DOC_ORDER = ["INVOICE", "DEBIT_NOTE", "CREDIT_NOTE", "PROFORMA", "PROTOCOL", "SALES_REPORT"]
  INVOICE_DOC_TYPES = {
    "INVOICE"      => {"Фактура", true},
    "DEBIT_NOTE"   => {"Дебитно известие", true},
    "CREDIT_NOTE"  => {"Кредитно известие", true},
    "PROFORMA"     => {"Проформа", false},
    "PROTOCOL"     => {"Протокол", true},
    "SALES_REPORT" => {"Отчет за продажби", true},
  }

  WH_LABELS = {
    "RECEIPT"         => "Приход",
    "PROD_RECEIPT"    => "Производствен приход",
    "SALES_ISSUE"     => "Продажба от склад",
    "SALES_RETURN"    => "Връщане от клиент",
    "PURCHASE_RETURN" => "Връщане към доставчик",
    "ISSUE"           => "Разход",
    "PROD_ISSUE"      => "Производствен разход",
    "TRANSFER"        => "Прехвърляне",
    "INV_PLUS"        => "Инвентаризация +",
    "INV_MINUS"       => "Инвентаризация −",
    "WRITE_OFF"       => "Брак",
    "EXPIRY"          => "Изтекъл срок",
    "OTHER"           => "Друг складов",
  }

  class Error < Exception
  end

  def self.format_number(number : Int64) : String
    return number.to_s if number < 0 || number > MAX_NUMBER
    "%010d" % number
  end

  def self.initialize_for_company(company : Company) : Nil
    return if DocumentNumberSequenceQuery.new.company_id(company.id).first?

    sales = create_seq(company, "SALES_TAX", "Данъчни продажби", true)
    proforma = create_seq(company, "PROFORMA", "Проформи", false)
    protocols = create_seq(company, "PROTOCOLS", "Протоколи и отчети", true)
    wh = create_seq(company, "WH_INTERNAL", "Вътрешни складови документи", false)

    start = company.invoice_next_number
    if start > 1
      SaveDocumentNumberSequence.update!(sales, current_number: start - 1)
    end

    bind(company, sales, "invoice_sale", "INVOICE")
    bind(company, sales, "invoice_sale", "DEBIT_NOTE")
    bind(company, sales, "invoice_sale", "CREDIT_NOTE")
    bind(company, proforma, "invoice_sale", "PROFORMA")
    bind(company, protocols, "invoice_sale", "PROTOCOL")
    bind(company, protocols, "invoice_sale", "SALES_REPORT")
    WH_TYPES.each { |t| bind(company, wh, "warehouse", t) }
  end

  def self.preview(sequence : DocumentNumberSequence) : String
    sequence.formatted(sequence.current_number + 1)
  end

  def self.allocate!(sequence_id : Int64) : String
    formatted = ""
    AppDatabase.transaction do
      seq = DocumentNumberSequenceQuery.new.id(sequence_id).for_update.first
      raise "sequence not found" unless seq
      nxt = seq.current_number + 1
      raise Overflow.new("number overflow") if nxt >= 10_000_000_000
      SaveDocumentNumberSequence.update!(seq, current_number: nxt)
      formatted = seq.formatted(nxt)
    end
    formatted
  rescue ex : PQ::PQError
    if ex.message.to_s.includes?("lock") || ex.message.to_s.includes?("deadlock")
      raise LockTimeout.new("number_lock_timeout")
    end
    raise ex
  end

  def self.sequence_for(company_id : Int64, family : String, document_type : String) : DocumentNumberSequence?
    binding = find_binding(company_id, family, document_type)
    if binding.nil? && family == "invoice_sale" && ["PROTOCOL", "SALES_REPORT"].includes?(document_type)
      ensure_protocol_sequence(company_id)
      binding = find_binding(company_id, family, document_type)
    end
    return nil unless binding
    DocumentNumberSequenceQuery.new.id(binding.document_number_sequence_id).first?
  end

  # Firms created before protocol numbering existed still need a chain.
  def self.ensure_protocol_sequence(company_id : Int64) : Nil
    company = CompanyQuery.new.find(company_id)
    initialize_for_company(company)
    seq = DocumentNumberSequenceQuery.new.company_id(company_id).code("PROTOCOLS").first? ||
      create_seq(company, "PROTOCOLS", "Протоколи и отчети", true)
    ["PROTOCOL", "SALES_REPORT"].each do |kind|
      next if find_binding(company_id, "invoice_sale", kind)
      bind(company, seq, "invoice_sale", kind)
    end
  end

  private def self.find_binding(company_id, family, document_type)
    DocumentSequenceBindingQuery.new
      .company_id(company_id)
      .family(family)
      .document_type(document_type)
      .first?
  end

  def self.ensure_invoice_defaults(company : Company) : Nil
    initialize_for_company(company)
    DocumentNumberSequenceQuery.new.company_id(company.id).each do |seq|
      SaveDocumentNumberSequence.update!(seq, padding: PADDING) if seq.padding != PADDING
    end
    ensure_kind(company, "INVOICE", "SALES_TAX", "Данъчни продажби", true)
    ensure_kind(company, "DEBIT_NOTE", "SALES_TAX", "Данъчни продажби", true)
    ensure_kind(company, "CREDIT_NOTE", "SALES_TAX", "Данъчни продажби", true)
    ensure_kind(company, "PROFORMA", "PROFORMA", "Проформи", false)
    ensure_kind(company, "PROTOCOL", "PROTOCOLS", "Протоколи и отчети", true)
    ensure_kind(company, "SALES_REPORT", "PROTOCOLS", "Протоколи и отчети", true)
  end

  def self.list_for(company : Company)
    ensure_invoice_defaults(company)
    DocumentNumberSequenceQuery.new.company_id(company.id).to_a.sort_by { |seq| {sort_key(seq), seq.id} }
  end

  def self.catalog
    INVOICE_DOC_ORDER.map do |code|
      pair = INVOICE_DOC_TYPES[code]
      {code: code, name: pair[0], tax: pair[1]}
    end
  end

  def self.serialize(seq : DocumentNumberSequence)
    bindings = DocumentSequenceBindingQuery.new.document_number_sequence_id(seq.id).to_a
    bindings.sort_by! { |b| type_sort(b) }
    issued = max_issued_number(seq.id)
    docs = InvoiceQuery.new.sequence_id(seq.id).select_count + WarehouseDocumentQuery.new.sequence_id(seq.id).select_count
    {
      id:             seq.id,
      code:           seq.code,
      name:           seq.name,
      current_number: seq.current_number,
      padding:        PADDING,
      is_tax:         seq.is_tax,
      next_number:    seq.current_number + 1,
      next_formatted: format_number(seq.current_number + 1),
      issued_max:     issued,
      deletable:      bindings.empty? && docs == 0_i64,
      document_types: bindings.map { |b|
        {
          family:        b.family,
          document_type: b.document_type,
          name:          type_label(b.family, b.document_type),
        }
      },
    }
  end

  def self.create_interval!(company : Company, name : String, next_number : Int64, document_types : Array(String)) : DocumentNumberSequence
    clean = name.strip
    raise Error.new("Въведете име на интервала") if clean.blank?
    raise Error.new("Името е прекалено дълго") if clean.size > 255
    raise Error.new("Изберете поне един вид документ") if document_types.empty?
    raise Error.new("Непознат вид документ") unless document_types.all? { |t| INVOICE_DOC_TYPES.has_key?(t) }
    raise Error.new("Номерът трябва да е между 1 и 9999999999") if next_number < 1 || next_number > MAX_NUMBER

    ensure_invoice_defaults(company)
    code = "RANGE_#{Time.utc.to_unix_ms}"
    tax = document_types.any? { |kind| kind != "PROFORMA" }
    saved_id = 0_i64
    AppDatabase.transaction do
      seq = create_seq(company, code, clean, tax, next_number - 1)
      document_types.each { |kind| move_binding(company, seq, kind) }
      refresh_tax_flags(company.id)
      saved_id = seq.id
    end
    DocumentNumberSequenceQuery.new.id(saved_id).first.not_nil!
  end

  def self.update_interval!(company_id : Int64, sequence_id : Int64, name : String, next_number : Int64) : DocumentNumberSequence
    clean = name.strip
    raise Error.new("Въведете име на интервала") if clean.blank?
    raise Error.new("Името е прекалено дълго") if clean.size > 255
    raise Error.new("Номерът трябва да е между 1 и 9999999999") if next_number < 1 || next_number > MAX_NUMBER
    found = DocumentNumberSequenceQuery.new.id(sequence_id).company_id(company_id).first?
    raise Error.new("Интервалът не е намерен") unless found
    seq = found.not_nil!
    issued = max_issued_number(seq.id)
    if next_number <= issued
      raise Error.new("Следващият номер трябва да е след последния издаден #{format_number(issued)}")
    end
    kinds = invoice_kinds(seq.id)
    tax = kinds.empty? ? seq.is_tax : kinds.any? { |kind| kind != "PROFORMA" }
    SaveDocumentNumberSequence.update!(seq,
      name: clean,
      current_number: next_number - 1,
      padding: PADDING,
      is_tax: tax
    )
  end

  def self.delete_interval!(company_id : Int64, sequence_id : Int64) : Nil
    found = DocumentNumberSequenceQuery.new.id(sequence_id).company_id(company_id).first?
    raise Error.new("Интервалът не е намерен") unless found
    seq = found.not_nil!
    unless DocumentSequenceBindingQuery.new.document_number_sequence_id(seq.id).to_a.empty?
      raise Error.new("Интервалът още има документи. Преместете ги в друг интервал.")
    end
    if InvoiceQuery.new.sequence_id(seq.id).select_count > 0_i64 || WarehouseDocumentQuery.new.sequence_id(seq.id).select_count > 0_i64
      raise Error.new("По този интервал има издадени документи и не може да се изтрие")
    end
    seq.delete
  end

  private def self.create_seq(company, code, name, is_tax, current_number : Int64 = 0_i64)
    SaveDocumentNumberSequence.create!(
      company_id: company.id,
      code: code,
      name: name,
      current_number: current_number,
      padding: PADDING,
      is_tax: is_tax,
      reset_policy: "never"
    )
  end

  private def self.bind(company, seq, family, document_type)
    SaveDocumentSequenceBinding.create!(
      company_id: company.id,
      family: family,
      document_type: document_type,
      document_number_sequence_id: seq.id
    )
  end

  private def self.ensure_kind(company : Company, kind : String, code : String, name : String, is_tax : Bool) : Nil
    return if find_binding(company.id, "invoice_sale", kind)
    seq = DocumentNumberSequenceQuery.new.company_id(company.id).code(code).first? ||
      create_seq(company, code, name, is_tax)
    bind(company, seq, "invoice_sale", kind)
  end

  private def self.sort_key(seq : DocumentNumberSequence) : Int32
    case seq.code
    when "SALES_TAX"   then 0
    when "PROFORMA"    then 1
    when "PROTOCOLS"   then 2
    when "WH_INTERNAL" then 8
    else                    3
    end
  end

  private def self.type_sort(binding : DocumentSequenceBinding) : Int32
    if idx = INVOICE_DOC_ORDER.index(binding.document_type)
      return idx
    end
    100
  end

  private def self.type_label(family : String, document_type : String) : String
    if family == "invoice_sale"
      if pair = INVOICE_DOC_TYPES[document_type]?
        return pair[0]
      end
    end
    WH_LABELS[document_type]? || document_type
  end

  private def self.invoice_kinds(sequence_id : Int64) : Array(String)
    kinds = [] of String
    DocumentSequenceBindingQuery.new
      .document_number_sequence_id(sequence_id)
      .family("invoice_sale")
      .each { |binding| kinds << binding.document_type }
    kinds
  end

  private def self.move_binding(company : Company, seq : DocumentNumberSequence, kind : String) : Nil
    if binding = find_binding(company.id, "invoice_sale", kind)
      SaveDocumentSequenceBinding.update!(binding, document_number_sequence_id: seq.id)
    else
      bind(company, seq, "invoice_sale", kind)
    end
  end

  private def self.refresh_tax_flags(company_id : Int64) : Nil
    DocumentNumberSequenceQuery.new.company_id(company_id).each do |seq|
      kinds = invoice_kinds(seq.id)
      next if kinds.empty?
      tax = kinds.any? { |kind| kind != "PROFORMA" }
      if seq.is_tax != tax || seq.padding != PADDING
        SaveDocumentNumberSequence.update!(seq, is_tax: tax, padding: PADDING)
      end
    end
  end

  private def self.max_issued_number(sequence_id : Int64) : Int64
    max = 0_i64
    InvoiceQuery.new.sequence_id(sequence_id).each do |inv|
      raw = inv.invoice_number
      next if raw.nil?
      if n = raw.not_nil!.to_i64?
        max = n if n > max
      end
    end
    WarehouseDocumentQuery.new.sequence_id(sequence_id).each do |doc|
      raw = doc.document_number
      next if raw.nil?
      if n = raw.not_nil!.to_i64?
        max = n if n > max
      end
    end
    max
  end
end
