require "./controlisy_parser"

# Imports parsed Controlisy data into the database
class ControlisyImporter
  struct ImportResult
    property success : Bool
    property contractors_created : Int32
    property contractors_updated : Int32
    property documents_imported : Int32
    property journal_entries_created : Int32
    property vat_journal_entries_created : Int32
    property errors : Array(String)
    property warnings : Array(String)

    def initialize(
      @success = true,
      @contractors_created = 0,
      @contractors_updated = 0,
      @documents_imported = 0,
      @journal_entries_created = 0,
      @vat_journal_entries_created = 0,
      @errors = [] of String,
      @warnings = [] of String
    )
    end
  end

  def initialize(@company_id : Int64)
  end

  # Convert VAT month from Controlisy format (DD.MM.YYYY) to YYYY-MM format
  private def convert_vat_period(vat_month : String) : String?
    return nil if vat_month.empty?

    # Try DD.MM.YYYY format first (e.g., "01.01.2025")
    if vat_month.includes?(".")
      parts = vat_month.split(".")
      if parts.size >= 3
        day = parts[0]
        month = parts[1].rjust(2, '0')
        year = parts[2]
        return "#{year}-#{month}"
      end
    end

    # Try YYYY-MM-DD or YYYY-MM format
    if vat_month.includes?("-")
      parts = vat_month.split("-")
      if parts.size >= 2
        return "#{parts[0]}-#{parts[1].rjust(2, '0')}"
      end
    end

    # Return as-is if already in correct format or unknown
    vat_month
  end

  # Import from XML content
  def import(xml_content : String) : ImportResult
    result = ImportResult.new

    begin
      parsed = ControlisyParser.parse(xml_content)

      # Debug info about parsing
      docs_with_accountings = parsed.documents.count { |d| !d.accountings.empty? }
      docs_with_vat = parsed.documents.count { |d| d.vat_data }

      result.warnings << "Парсване: #{parsed.contractors.size} контрагента, #{parsed.documents.size} документа (#{docs_with_accountings} със счетоводство, #{docs_with_vat} с ДДС данни)"

      # Import contractors
      import_contractors(parsed.contractors, result)

      # Import documents and journal entries
      import_documents(parsed.documents, result)

      result
    rescue ex
      result.success = false
      result.errors << "Parse error: #{ex.message}"
      result
    end
  end

  private def import_contractors(contractors : Array(ControlisyParser::Contractor), result : ImportResult)
    contractors.each do |c|
      if c.name.empty?
        result.warnings << "Контрагент с ЕИК #{c.eik}: пропуснат - липсва име"
        next
      end

      # Try to find existing counterpart by EIK or VAT number
      existing = find_counterpart(c.eik, c.vat_number)

      if existing
        # Update if needed
        update_counterpart(existing, c)
        result.contractors_updated += 1
      else
        # Create new
        create_counterpart(c)
        result.contractors_created += 1
      end
    rescue ex
      result.errors << "Contractor #{c.name}: #{ex.message}"
    end
  end

  private def find_counterpart(eik : String, vat_number : String) : Counterpart?
    return nil if eik.empty? && vat_number.empty?

    query = CounterpartQuery.new.company_id(@company_id)

    if !eik.empty?
      found = query.eik(eik).first?
      return found if found
    end

    if !vat_number.empty?
      found = CounterpartQuery.new.company_id(@company_id).vat_number(vat_number).first?
      return found if found
    end

    nil
  end

  private def create_counterpart(c : ControlisyParser::Contractor)
    SaveCounterpart.create!(
      company_id: @company_id,
      name: c.name,
      eik: c.eik.empty? ? nil : c.eik,
      vat_number: c.vat_number.empty? ? nil : c.vat_number,
      counterpart_type: "both",
      is_active: true
    )
  end

  private def update_counterpart(existing : Counterpart, c : ControlisyParser::Contractor)
    # Only update if we have more data
    needs_update = false
    updates = {} of Symbol => String

    if existing.vat_number.nil? && !c.vat_number.empty?
      needs_update = true
    end

    # For now, we don't update existing counterparts
    # This could be expanded later
  end

  private def import_documents(documents : Array(ControlisyParser::Document), result : ImportResult)
    documents.each do |doc|
      import_document(doc, result)
    end
  end

  private def import_document(doc : ControlisyParser::Document, result : ImportResult)
    # Skip if no accountings
    if doc.accountings.empty?
      result.warnings << "Документ #{doc.document_number}: пропуснат - няма счетоводни записи"
      return
    end

    # Detect if this is a payment document (not an invoice)
    # Payment documents have RelDoc, no VATData, and reason contains "Разплащане" or similar
    is_payment = is_payment_document?(doc)

    # Generate unique reference key
    # Payments use different suffix to avoid collision with invoice that has same ca_doc_id
    reference_key = if is_payment && doc.rel_doc
                      "CTRL-#{doc.ca_doc_id}-PAY"
                    else
                      "CTRL-#{doc.ca_doc_id}"
                    end

    # Skip if already imported (dedup by reference)
    existing = JournalEntryQuery.new
      .company_id(@company_id)
      .reference(reference_key)
      .first?

    if existing
      skipped_count = result.warnings.count { |w| w.includes?("вече импортиран") }
      result.warnings << "Документ #{doc.document_number}: пропуснат - вече импортиран"
      return
    end

    # Parse dates
    doc_date = parse_date(doc.document_date)
    unless doc_date
      result.warnings << "Документ #{doc.document_number}: пропуснат - невалидна дата '#{doc.document_date}'"
      return
    end

    # Find or create counterpart for this document
    counterpart_id = find_document_counterpart_id(doc)

    # Determine entry type based on VAT register
    entry_type = determine_entry_type(doc)

    # Build journal lines from accountings
    lines = build_journal_lines(doc)
    if lines.empty?
      result.warnings << "Документ #{doc.document_number}: пропуснат - не може да се изградят счетоводни редове (липсват дебит/кредит двойки)"
      return
    end

    # Determine VAT operations from VAT data for journal entry classification
    # Payment documents have no VAT
    vat_purchase_op = nil
    vat_sales_op = nil
    vat_period_value = nil
    description = doc.reason || "Controlisy import"

    if is_payment
      # For payments: no VAT, generate proper description based on payment type
      payment_info = detect_payment_type(doc)
      description = payment_info[:description]
      # Payment documents don't have VAT period
      vat_period_value = nil
    else
      # For invoices: set VAT operations
      vat_period_value = convert_vat_period(doc.vat_month)
      if vat_data = doc.vat_data
        if !vat_data.entries.empty?
          op_iden = extract_vat_op_iden(vat_data.entries.first.vat_operation_iden)

          if vat_data.purchases?
            vat_purchase_op = map_purchase_op_label(op_iden)
          elsif vat_data.sales?
            vat_sales_op = map_sales_op_label(op_iden)
          end
        end
      end
    end

    # Create journal entry
    journal_entry = SaveJournalEntry.create!(
      company_id: @company_id,
      entry_date: doc_date,
      document_date: doc_date,
      document_number: doc.document_number,
      description: description,
      reference: reference_key,
      status: "draft",
      vat_period: vat_period_value,
      total_amount: doc.total_amount_bgn.to_f,
      total_vat_amount: is_payment ? 0.0 : doc.vat_amount_bgn.to_f,
      counterpart_id: counterpart_id,
      vat_purchase_operation: vat_purchase_op,
      vat_sales_operation: vat_sales_op,
      lines: JSON.build { |json| json.array { lines.each { |l| l.to_json(json) } } }
    )

    result.documents_imported += 1
    result.journal_entries_created += 1

    # Also create VAT journal entry if document has VAT data
    if vat_data = doc.vat_data
      if !vat_data.entries.empty? && !doc.vat_month.empty?
        create_vat_journal_entry(doc, vat_data, doc_date, journal_entry.id, result)
      end
    end
  rescue ex
    result.errors << "Document #{doc.document_number}: #{ex.message}"
  end

  # Create VAT journal entry from Controlisy document
  private def create_vat_journal_entry(
    doc : ControlisyParser::Document,
    vat_data : ControlisyParser::VatData,
    doc_date : Time,
    journal_entry_id : Int64,
    result : ImportResult
  )
    # Convert and parse VAT period
    vat_period = convert_vat_period(doc.vat_month)
    return if vat_period.nil? || vat_period.empty?

    period_parts = vat_period.split("-")
    return if period_parts.size < 2

    period_year = period_parts[0].to_i
    period_month = period_parts[1].to_i

    # Find or create VAT return for this period
    vat_return = find_or_create_vat_return(period_year, period_month, result)
    return unless vat_return

    # Determine entry type
    entry_type = vat_data.purchases? ? "purchase" : "sales"

    # Get document type from doc_ident (default to "01" - Invoice)
    doc_type = (doc.doc_ident || "01").strip
    doc_type = "01" if doc_type.empty?

    # Pad single digit to two digits (1 -> 01, 2 -> 02, etc.)
    if doc_type.size == 1 && doc_type[0].ascii_number?
      doc_type = "0#{doc_type}"
    end

    # Validate document type - must be in NAP allowed list
    valid_types = if entry_type == "purchase"
      VatJournalEntry::PURCHASE_DOC_TYPES.keys
    else
      VatJournalEntry::SALES_DOC_TYPES.keys
    end

    unless valid_types.includes?(doc_type)
      # Try to normalize common mappings
      doc_type = case doc_type
      when "1", "Ф", "ф", "F", "f", "invoice", "Invoice", "INVOICE"
        "01"
      when "2", "ДИ", "ди", "DN", "dn", "debit", "Debit"
        "02"
      when "3", "КИ", "ки", "CN", "cn", "credit", "Credit"
        "03"
      when "5"
        "05"
      when "7"
        "07"
      when "9", "П", "п", "protocol", "Protocol"
        "09"
      else
        # Default to invoice if unknown
        result.warnings << "Документ #{doc.document_number}: непознат тип '#{doc.doc_ident}', използван '01' (Фактура)"
        "01"
      end
    end

    # Map VAT operation to appropriate columns
    amounts = map_vat_operation_to_amounts(vat_data, entry_type)

    # Skip if VAT operation indicates "does not enter register" (op_iden = "0")
    if amounts[:skip_vat_entry]
      result.warnings << "Документ #{doc.document_number}: ДДС операция '0' - не влиза в дневник, пропуснат ДДС запис"
      return
    end

    # Get next row number
    row_number = get_next_row_number(vat_return.id, entry_type)

    # Determine document type for protocol operations (9001, 9002)
    first_op = extract_vat_op_iden(vat_data.entries.first?.try(&.vat_operation_iden))
    if first_op == "9001" || first_op == "9002"
      doc_type = "09"  # Protocol
    end

    # Create VAT journal entry
    SaveVatJournalEntry.create!(
      company_id: @company_id,
      vat_return_id: vat_return.id,
      row_number: row_number,
      document_type: doc_type,
      document_number: doc.document_number,
      document_date: doc_date,
      counterpart_vat: vat_data.contractor_vat_number,
      counterpart_name: vat_data.contractor_name,
      goods_description: (doc.reason || "")[0, 30],
      delivery_code: amounts[:delivery_code],
      entry_type: entry_type,
      # Purchase amounts
      base_no_credit: amounts[:base_no_credit],
      base_full_credit: amounts[:base_full_credit],
      vat_full_credit: amounts[:vat_full_credit],
      base_partial_credit: amounts[:base_partial_credit],
      vat_partial_credit: amounts[:vat_partial_credit],
      vop_base: amounts[:vop_base],
      vop_vat: amounts[:vop_vat],
      base_20: amounts[:base_20],
      vat_20: amounts[:vat_20],
      base_9: amounts[:base_9],
      vat_9: amounts[:vat_9],
      # Sales amounts
      total_base: amounts[:total_base],
      total_vat: amounts[:total_vat],
      sales_base_20: amounts[:sales_base_20],
      sales_vat_20: amounts[:sales_vat_20],
      sales_base_vop: amounts[:sales_base_vop],
      sales_vat_vop: amounts[:sales_vat_vop],
      sales_base_9: amounts[:sales_base_9],
      sales_vat_9: amounts[:sales_vat_9],
      sales_base_0_chapter3: amounts[:sales_base_0],
      sales_base_vod: amounts[:sales_base_vod],
      sales_base_0_articles: amounts[:sales_base_0_articles],
      sales_base_services_21: amounts[:sales_base_services_21],
      sales_base_69_2: amounts[:sales_base_69_2],
      sales_base_exempt: amounts[:sales_base_exempt],
      source_journal_entry_id: journal_entry_id,
      notes: "Imported from Controlisy (ca_doc_id: #{doc.ca_doc_id})"
    )

    result.vat_journal_entries_created += 1
  rescue ex
    result.warnings << "VAT journal for #{doc.document_number}: #{ex.message}"
  end

  # Find or create VAT return for period
  private def find_or_create_vat_return(year : Int32, month : Int32, result : ImportResult) : VatReturn?
    # Try to find existing
    existing = VatReturnQuery.new
      .company_id(@company_id)
      .period_year(year)
      .period_month(month)
      .first?

    return existing if existing

    # Create new VAT return
    vat_return = SaveVatReturn.create!(
      company_id: @company_id,
      period_year: year,
      period_month: month,
      status: "draft"
    )

    result.warnings << "Created VAT return for #{year}-#{month.to_s.rjust(2, '0')}"
    vat_return
  rescue ex
    result.warnings << "Could not create VAT return: #{ex.message}"
    nil
  end

  # Get next row number for entry type
  private def get_next_row_number(vat_return_id : Int64, entry_type : String) : Int32
    max = VatJournalEntryQuery.new
      .vat_return_id(vat_return_id)
      .entry_type(entry_type)
      .select_count

    max.to_i + 1
  end

  # Extract clean numeric VAT operation identifier from raw XML value
  # Handles cases like "5 – Доставка на услуги..." -> "5", "01" -> "1", "9001" -> "9001"
  private def extract_vat_op_iden(raw : String?) : String
    return "" if raw.nil? || raw.empty?
    stripped = raw.strip
    if match = stripped.match(/^(\d+)/)
      match[1].lstrip('0').empty? ? "0" : match[1].lstrip('0')
    else
      ""
    end
  end

  # Map VAT operation identifier to amount columns
  # Uses vatOperationIden codes from Controlisy documentation v2:
  #   Purchases: 0(skip), 1(full credit), 2(partial credit), 3(no VAT), 5(trilateral), 6(no credit right)
  #   Sales: 0(skip), 1(20%), 2(9%), 3(0% ch3), 4(VOD), 5(services Art.21), 6(Art.69§2),
  #          7(distance), 8(exempt), 9(trilateral), 10(Art.140/146/173)
  #   Protocol: 9001(VOP), 9002(Art.82)
  private def map_vat_operation_to_amounts(vat_data : ControlisyParser::VatData, entry_type : String) : NamedTuple(
    base_no_credit: Float64,
    base_full_credit: Float64,
    vat_full_credit: Float64,
    base_partial_credit: Float64,
    vat_partial_credit: Float64,
    vop_base: Float64,
    vop_vat: Float64,
    base_20: Float64,
    vat_20: Float64,
    base_9: Float64,
    vat_9: Float64,
    total_base: Float64,
    total_vat: Float64,
    sales_base_20: Float64,
    sales_vat_20: Float64,
    sales_base_vop: Float64,
    sales_vat_vop: Float64,
    sales_base_9: Float64,
    sales_vat_9: Float64,
    sales_base_0: Float64,
    sales_base_vod: Float64,
    sales_base_0_articles: Float64,
    sales_base_services_21: Float64,
    sales_base_69_2: Float64,
    sales_base_exempt: Float64,
    delivery_code: String?,
    skip_vat_entry: Bool
  )
    amounts = {
      base_no_credit:         0.0,
      base_full_credit:       0.0,
      vat_full_credit:        0.0,
      base_partial_credit:    0.0,
      vat_partial_credit:     0.0,
      vop_base:               0.0,
      vop_vat:                0.0,
      base_20:                0.0,
      vat_20:                 0.0,
      base_9:                 0.0,
      vat_9:                  0.0,
      total_base:             0.0,
      total_vat:              0.0,
      sales_base_20:          0.0,
      sales_vat_20:           0.0,
      sales_base_vop:         0.0,
      sales_vat_vop:          0.0,
      sales_base_9:           0.0,
      sales_vat_9:            0.0,
      sales_base_0:           0.0,
      sales_base_vod:         0.0,
      sales_base_0_articles:  0.0,
      sales_base_services_21: 0.0,
      sales_base_69_2:        0.0,
      sales_base_exempt:      0.0,
      delivery_code:          nil.as(String?),
      skip_vat_entry:         false,
    }

    vat_data.entries.each do |entry|
      base = entry.tax_base.to_f
      vat = entry.vat_amount_bgn.to_f
      rate = entry.vat_rate.to_f
      op_iden = extract_vat_op_iden(entry.vat_operation_iden)
      add_iden = extract_vat_op_iden(entry.vat_operation_additional_iden)
      detail = (entry.vat_operation_detail || "").strip

      # Map vatOperationDetail to delivery_code (Приложение 2)
      if !detail.empty?
        dc = case detail
             when "1" then "01"  # Part I Appendix 2
             when "2" then "02"  # Part II Appendix 2
             else nil
             end
        amounts = amounts.merge({delivery_code: dc}) if dc
      end

      # Map vatOperationAdditionalIden to delivery_code
      if amounts[:delivery_code].nil? && !add_iden.empty?
        dc = case add_iden
             when "4" then "41"  # Dispatch goods warehouse mode
             when "5" then "48"  # Return goods warehouse mode
             when "6" then "43"  # Person replacement Art.15a
             else nil
             end
        amounts = amounts.merge({delivery_code: dc}) if dc
      end

      if entry_type == "purchase"
        case op_iden
        when "0"
          # Does not enter register - skip VatJournalEntry
          amounts = amounts.merge({skip_vat_entry: true})
        when "1"
          # Full VAT credit
          amounts = amounts.merge({base_full_credit: amounts[:base_full_credit] + base})
          amounts = amounts.merge({vat_full_credit: amounts[:vat_full_credit] + vat})
          if rate >= 8.0 && rate < 15.0
            amounts = amounts.merge({base_9: amounts[:base_9] + base})
            amounts = amounts.merge({vat_9: amounts[:vat_9] + vat})
          else
            amounts = amounts.merge({base_20: amounts[:base_20] + base})
            amounts = amounts.merge({vat_20: amounts[:vat_20] + vat})
          end
        when "2"
          # Partial VAT credit
          amounts = amounts.merge({base_partial_credit: amounts[:base_partial_credit] + base})
          amounts = amounts.merge({vat_partial_credit: amounts[:vat_partial_credit] + vat})
        when "3"
          # No VAT (base without credit)
          amounts = amounts.merge({base_no_credit: amounts[:base_no_credit] + base})
        when "5"
          # Trilateral intermediary acquisition - treated like VOP
          amounts = amounts.merge({vop_base: amounts[:vop_base] + base})
          amounts = amounts.merge({vop_vat: amounts[:vop_vat] + vat})
        when "6"
          # No credit right (NEW in v2) - base without credit
          amounts = amounts.merge({base_no_credit: amounts[:base_no_credit] + base})
        when "9001"
          # VOP protocol - intra-EU acquisition
          amounts = amounts.merge({vop_base: amounts[:vop_base] + base})
          amounts = amounts.merge({vop_vat: amounts[:vop_vat] + vat})
        when "9002"
          # Art.82 protocol - reverse charge, full credit
          amounts = amounts.merge({base_full_credit: amounts[:base_full_credit] + base})
          amounts = amounts.merge({vat_full_credit: amounts[:vat_full_credit] + vat})
          amounts = amounts.merge({base_20: amounts[:base_20] + base})
          amounts = amounts.merge({vat_20: amounts[:vat_20] + vat})
        else
          # Fallback based on VAT amount
          if vat > 0
            amounts = amounts.merge({base_full_credit: amounts[:base_full_credit] + base})
            amounts = amounts.merge({vat_full_credit: amounts[:vat_full_credit] + vat})
          else
            amounts = amounts.merge({base_no_credit: amounts[:base_no_credit] + base})
          end
        end
      else # sales
        if op_iden == "0"
          # Does not enter register
          amounts = amounts.merge({skip_vat_entry: true})
          next
        end

        amounts = amounts.merge({total_base: amounts[:total_base] + base})
        amounts = amounts.merge({total_vat: amounts[:total_vat] + vat})

        case op_iden
        when "1"
          # Taxable at 20%
          amounts = amounts.merge({sales_base_20: amounts[:sales_base_20] + base})
          amounts = amounts.merge({sales_vat_20: amounts[:sales_vat_20] + vat})
        when "2"
          # Taxable at 9%
          amounts = amounts.merge({sales_base_9: amounts[:sales_base_9] + base})
          amounts = amounts.merge({sales_vat_9: amounts[:sales_vat_9] + vat})
        when "3"
          # 0% per Chapter 3
          amounts = amounts.merge({sales_base_0: amounts[:sales_base_0] + base})
        when "4"
          # Intra-EU delivery (VOD)
          amounts = amounts.merge({sales_base_vod: amounts[:sales_base_vod] + base})
        when "5"
          # Services Art.21 para 2
          amounts = amounts.merge({sales_base_services_21: amounts[:sales_base_services_21] + base})
        when "6"
          # Art.69 para 2
          amounts = amounts.merge({sales_base_69_2: amounts[:sales_base_69_2] + base})
        when "7"
          # Distance sales to another member state
          amounts = amounts.merge({sales_base_exempt: amounts[:sales_base_exempt] + base})
        when "8"
          # Exempt supplies
          amounts = amounts.merge({sales_base_exempt: amounts[:sales_base_exempt] + base})
        when "9"
          # Trilateral intermediary - treated as VOD
          amounts = amounts.merge({sales_base_vod: amounts[:sales_base_vod] + base})
        when "10"
          # Art.140, 146, 173
          amounts = amounts.merge({sales_base_0_articles: amounts[:sales_base_0_articles] + base})
        when "9001"
          # VOP protocol in sales register
          amounts = amounts.merge({sales_base_vop: amounts[:sales_base_vop] + base})
          amounts = amounts.merge({sales_vat_vop: amounts[:sales_vat_vop] + vat})
        when "9002"
          # Art.82 protocol in sales register
          amounts = amounts.merge({sales_base_vop: amounts[:sales_base_vop] + base})
          amounts = amounts.merge({sales_vat_vop: amounts[:sales_vat_vop] + vat})
        else
          # Fallback based on rate
          if rate >= 19.0
            amounts = amounts.merge({sales_base_20: amounts[:sales_base_20] + base})
            amounts = amounts.merge({sales_vat_20: amounts[:sales_vat_20] + vat})
          elsif rate >= 8.0
            amounts = amounts.merge({sales_base_9: amounts[:sales_base_9] + base})
            amounts = amounts.merge({sales_vat_9: amounts[:sales_vat_9] + vat})
          elsif vat == 0.0
            amounts = amounts.merge({sales_base_0: amounts[:sales_base_0] + base})
          else
            amounts = amounts.merge({sales_base_20: amounts[:sales_base_20] + base})
            amounts = amounts.merge({sales_vat_20: amounts[:sales_vat_20] + vat})
          end
        end
      end
    end

    amounts
  end

  # Map Controlisy purchase vatOperationIden to journal entry label
  # Format: "DOC_TYPE|LABEL" where DOC_TYPE is the default doc type and LABEL drives column mapping
  private def map_purchase_op_label(op_iden : String) : String?
    case op_iden
    when "0"    then nil  # Does not enter register
    when "1"    then "01|PURCHASE_CREDIT"       # Full credit 20%
    when "2"    then "01|PURCHASE_PARTIAL"       # Partial credit
    when "3"    then "01|PURCHASE_NO_CREDIT"     # No VAT
    when "5"    then "09|PURCHASE_VOP"           # Trilateral intermediary (protocol)
    when "6"    then "01|PURCHASE_NO_RIGHT"      # No credit right (v2)
    when "9001" then "09|ICA"                    # VOP protocol
    when "9002" then "09|PURCHASE_CREDIT"        # Art.82 protocol
    else             "01|PURCHASE_CREDIT"        # Default to full credit
    end
  end

  # Map Controlisy sales vatOperationIden to journal entry label
  private def map_sales_op_label(op_iden : String) : String?
    case op_iden
    when "0"    then nil  # Does not enter register
    when "1"    then "01|SALES_20"               # 20%
    when "2"    then "01|SALES_COL_17"           # 9%
    when "3"    then "01|EXPORT_ZERO"            # 0% Chapter 3
    when "4"    then "01|ICS"                    # VOD (intra-EU delivery)
    when "5"    then "01|SALES_COL_22"           # Services Art.21 para 2
    when "6"    then "01|SALES_COL_23"           # Art.69 para 2
    when "7"    then "01|SALES_COL_24"           # Distance sales
    when "8"    then "01|SALES_COL_24"           # Exempt supplies
    when "9"    then "01|ICS"                    # Trilateral intermediary
    when "10"   then "01|SALES_COL_21"           # Art.140, 146, 173
    when "9001" then "09|SALES_VOP"              # VOP protocol in sales
    when "9002" then "09|SALES_VOP"              # Art.82 protocol in sales
    else             "01|SALES_20"               # Default to 20%
    end
  end

  private def find_document_counterpart_id(doc : ControlisyParser::Document) : Int64?
    # Try to find from VAT data first
    if vat_data = doc.vat_data
      if vat_number = vat_data.contractor_vat_number
        if counterpart = CounterpartQuery.new.company_id(@company_id).vat_number(vat_number).first?
          return counterpart.id
        end
      end
    end

    # Try from accounting details
    doc.accountings.each do |acc|
      acc.details.each do |detail|
        if vat_number = detail.contractor_vat_number
          if counterpart = CounterpartQuery.new.company_id(@company_id).vat_number(vat_number).first?
            return counterpart.id
          end
        end
        if eik = detail.contractor_eik
          if counterpart = CounterpartQuery.new.company_id(@company_id).eik(eik).first?
            return counterpart.id
          end
        end
      end
    end

    nil
  end

  private def determine_entry_type(doc : ControlisyParser::Document) : String
    if vat_data = doc.vat_data
      return "purchase" if vat_data.purchases?
      return "sale" if vat_data.sales?
    end

    # Check account numbers
    doc.accountings.each do |acc|
      acc.details.each do |detail|
        case detail.account_number
        when /^401/
          return "purchase" if detail.credit?
        when /^411/
          return "sale" if detail.debit?
        when /^50/
          return "bank"
        end
      end
    end

    "other"
  end

  private def build_journal_lines(doc : ControlisyParser::Document) : Array(JSON::Any)
    lines = [] of JSON::Any
    line_num = 1

    doc.accountings.each do |acc|
      debit = acc.debit_detail
      credit = acc.credit_detail

      next unless debit && credit

      # Find account IDs
      debit_account_id = find_or_create_account(debit.account_number, debit.account_name)
      credit_account_id = find_or_create_account(credit.account_number, credit.account_name)

      debit_desc = debit.account_name.empty? ? (doc.reason || "") : debit.account_name
      credit_desc = credit.account_name.empty? ? (doc.reason || "") : credit.account_name

      # Build debit line with extended data
      debit_line = {
        "line_number"       => JSON::Any.new(line_num.to_i64),
        "account_id"        => debit_account_id ? JSON::Any.new(debit_account_id.to_i64) : JSON::Any.new(nil),
        "account_code"      => JSON::Any.new(debit.account_number),
        "description"       => JSON::Any.new(debit_desc),
        "debit"             => JSON::Any.new(acc.amount_bgn.to_f),
        "credit"            => JSON::Any.new(0.0),
      } of String => JSON::Any

      # Add currency info if present
      if currency = debit.currency
        debit_line["currency"] = JSON::Any.new(currency)
        if ca = debit.currency_amount
          debit_line["currency_amount"] = JSON::Any.new(ca.to_f)
        end
      end
      # Add counterpart info if present
      if cn = debit.contractor_name
        debit_line["counterpart_name"] = JSON::Any.new(cn)
      end
      if cv = debit.contractor_vat_number
        debit_line["counterpart_vat"] = JSON::Any.new(cv)
      end
      # Add account item1 (future article number)
      if item1 = debit.account_item1
        debit_line["account_item1"] = JSON::Any.new(item1)
      end

      lines << JSON::Any.new(debit_line)
      line_num += 1

      # Build credit line with extended data
      credit_line = {
        "line_number"       => JSON::Any.new(line_num.to_i64),
        "account_id"        => credit_account_id ? JSON::Any.new(credit_account_id.to_i64) : JSON::Any.new(nil),
        "account_code"      => JSON::Any.new(credit.account_number),
        "description"       => JSON::Any.new(credit_desc),
        "debit"             => JSON::Any.new(0.0),
        "credit"            => JSON::Any.new(acc.amount_bgn.to_f),
      } of String => JSON::Any

      if currency = credit.currency
        credit_line["currency"] = JSON::Any.new(currency)
        if ca = credit.currency_amount
          credit_line["currency_amount"] = JSON::Any.new(ca.to_f)
        end
      end
      if cn = credit.contractor_name
        credit_line["counterpart_name"] = JSON::Any.new(cn)
      end
      if cv = credit.contractor_vat_number
        credit_line["counterpart_vat"] = JSON::Any.new(cv)
      end
      if item1 = credit.account_item1
        credit_line["account_item1"] = JSON::Any.new(item1)
      end

      lines << JSON::Any.new(credit_line)
      line_num += 1
    end

    lines
  end

  private def find_or_create_account(code : String, name : String) : Int64?
    return nil if code.empty?

    # Try to find existing account
    existing = AccountQuery.new.company_id(@company_id).code(code).first?
    return existing.id if existing

    # Determine account type from code
    account_type = determine_account_type(code)

    # Get name from SAFT accounts if empty
    account_name = name
    if account_name.empty?
      saft = SaftAccountQuery.new.by_code(code).first?
      account_name = saft.name if saft
    end
    account_name = "Account #{code}" if account_name.empty?

    # Create account
    account = SaveAccount.create!(
      company_id: @company_id,
      code: code,
      name: account_name,
      account_type: account_type,
      is_active: true
    )

    account.id
  end

  private def determine_account_type(code : String) : String
    first_digit = code[0]?
    case first_digit
    when '1'       # Capital and loans
      "equity"
    when '2'       # Fixed assets
      "asset"
    when '3'       # Inventory
      "asset"
    when '4'       # Settlements
      code.starts_with?("40") ? "liability" : "asset"  # 40x = suppliers (liability), 41x = customers (asset)
    when '5'       # Financial resources
      "asset"
    when '6'       # Expenses
      "expense"
    when '7'       # Revenue
      "revenue"
    when '9'       # Contingent
      "asset"
    else
      "asset"
    end
  end

  private def parse_date(date_str : String) : Time?
    return nil if date_str.empty?

    # Try different formats
    formats = [
      "%Y-%m-%d",
      "%d.%m.%Y",
      "%d/%m/%Y",
    ]

    formats.each do |fmt|
      begin
        return Time.parse(date_str, fmt, Time::Location::UTC)
      rescue
        next
      end
    end

    nil
  end

  # Detect payment type and generate appropriate description
  # Returns a named tuple with payment type and description
  private def detect_payment_type(doc : ControlisyParser::Document) : NamedTuple(type: String, description: String)
    payment_type = "unknown"
    is_expense = true  # Default: payment to supplier (РКО)

    # Check accounting details to determine payment type
    doc.accountings.each do |acc|
      if debit = acc.debit_detail
        # If debiting 401 (suppliers) = paying supplier (РКО)
        # If debiting 501/503/509 (cash/bank) = receiving payment (ПКО)
        if debit.account_number.starts_with?("401")
          is_expense = true
        elsif debit.account_number.starts_with?("50")
          is_expense = false  # Receiving payment
        end
      end

      if credit = acc.credit_detail
        case credit.account_number
        when /^501/  # Cash
          payment_type = "cash"
        when /^503/  # Bank
          payment_type = "bank"
        when /^509/  # Card payment buffers
          payment_type = "card"
        when /^411/  # Customer being paid
          is_expense = false
        end
      end
    end

    # Get contractor name from RelDoc if available
    contractor_name = ""
    if rel_doc = doc.rel_doc
      contractor_name = rel_doc.contractor_name || ""
    end
    # Fallback to accounting details
    if contractor_name.empty?
      doc.accountings.each do |acc|
        acc.details.each do |detail|
          if name = detail.contractor_name
            contractor_name = name
            break
          end
        end
        break unless contractor_name.empty?
      end
    end

    # Build description
    base_desc = case payment_type
                when "cash"
                  is_expense ? "РКО" : "ПКО"
                when "card"
                  "Плащане с карта"
                when "bank"
                  "Банково плащане"
                else
                  is_expense ? "РКО" : "ПКО"
                end

    # Include contractor and related document info
    description = base_desc
    description += " - #{contractor_name}" unless contractor_name.empty?
    if rel_doc = doc.rel_doc
      if doc_num = rel_doc.document_number
        description += " (ф-ра #{doc_num})"
      end
    end

    {type: payment_type, description: description}
  end

  # Detect if document is a payment (cash, card, COD) rather than an invoice
  # Payment documents in Controlisy have:
  # - RelDoc pointing to the related invoice
  # - No VATData section
  # - Reason contains "Разплащане" or similar
  # - Accounts like 501 (cash), 509xx (card buffers) in credit side
  private def is_payment_document?(doc : ControlisyParser::Document) : Bool
    # Has RelDoc - strong indicator of payment
    return false unless doc.rel_doc

    # No VAT data - payments don't have VAT
    return false if doc.vat_data

    # Check reason text for payment indicators
    reason = (doc.reason || "").downcase
    payment_keywords = ["разплащане", "плащане", "каса", "карта", "наложен платеж", "payment"]
    if payment_keywords.any? { |kw| reason.includes?(kw) }
      return true
    end

    # Check if accounting involves cash/bank accounts (50x) on credit side
    doc.accountings.each do |acc|
      if credit = acc.credit_detail
        # 501 = cash, 503 = bank, 509xx = card payment buffers
        if credit.account_number.starts_with?("50")
          return true
        end
      end
    end

    # Check if debit side is clearing 401 (suppliers) or 411 (customers)
    doc.accountings.each do |acc|
      if debit = acc.debit_detail
        # 401 = suppliers being paid, 411 = receiving customer payment
        if debit.account_number.starts_with?("401") || debit.account_number.starts_with?("411")
          if credit = acc.credit_detail
            # And credit is cash/bank
            if credit.account_number.starts_with?("50")
              return true
            end
          end
        end
      end
    end

    false
  end
end
