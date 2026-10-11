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

      # Bank extracts often have an empty <Contractors> list; counterpart
      # data lives on RelDoc / AccountingDetail instead.
      contractors = merge_contractors(
        parsed.contractors,
        extract_contractors_from_documents(parsed.documents)
      )

      result.warnings << "Парсване: #{contractors.size} контрагента, #{parsed.documents.size} документа (#{docs_with_accountings} със счетоводство, #{docs_with_vat} с ДДС данни)"

      import_contractors(contractors, result)

      # Import documents and journal entries
      import_documents(parsed.documents, parsed.import_type, result)

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

      existing = find_counterpart_normalized(c.eik, c.vat_number, c.name)

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
    find_counterpart_normalized(eik, vat_number, "")
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

  private def import_documents(documents : Array(ControlisyParser::Document), import_type : String, result : ImportResult)
    documents.each do |doc|
      import_document(doc, import_type, result)
    end
  end

  private def import_document(doc : ControlisyParser::Document, import_type : String, result : ImportResult)
    # Skip if no accountings
    if doc.accountings.empty?
      result.warnings << "Документ #{doc.document_number}: пропуснат - няма счетоводни записи"
      return
    end

    # Bank/cash rows are not invoices: they have their own numbering and no VAT.
    # RelDoc (when present) only points at the invoice being paid.
    is_payment = is_payment_document?(doc, import_type)
    reference_key = controlisy_reference_key(doc, is_payment)

    if existing = find_existing_import(doc, reference_key, is_payment)
      if existing.counterpart_id.nil?
        if cid = find_or_create_document_counterpart(doc, result)
          patch_existing_counterpart(existing, cid)
          result.warnings << "Документ #{doc.document_number}: допълнен контрагент към вече импортиран запис"
          return
        end
      end
      result.warnings << "Документ #{doc.document_number}: пропуснат - вече импортиран"
      return
    end

    # Parse dates
    doc_date = parse_date(doc.document_date)
    unless doc_date
      result.warnings << "Документ #{doc.document_number}: пропуснат - невалидна дата '#{doc.document_date}'"
      return
    end

    # Counterpart is optional on bank rows. Attach it only when the XML has one.
    counterpart_id = find_or_create_document_counterpart(doc, result)

    # Build journal lines from accountings
    lines = build_journal_lines(doc, counterpart_id)
    if lines.empty?
      result.warnings << "Документ #{doc.document_number}: пропуснат - не може да се изградят счетоводни редове (липсват дебит/кредит двойки)"
      return
    end

    # Bank / cash / payment rows are ordinary NON_VAT journal entries.
    # Counterpart (when present) is only for 401/411 subledger — never VAT.
    ordinary = is_payment || import_type == "bank"

    vat_purchase_op = nil
    vat_sales_op = nil
    vat_period_value = nil
    vat_amount = 0.0
    description = doc.reason || "Controlisy import"

    if ordinary
      payment_info = detect_payment_type(doc)
      description = payment_info[:description]
    else
      vat_period_value = convert_vat_period(doc.vat_month)
      vat_amount = doc.vat_amount_bgn.to_f
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
      total_vat_amount: vat_amount,
      counterpart_id: counterpart_id,
      vat_purchase_operation: vat_purchase_op,
      vat_sales_operation: vat_sales_op,
      lines: JSON.build { |json| json.array { lines.each { |l| l.to_json(json) } } }
    )

    result.documents_imported += 1
    result.journal_entries_created += 1

    # VAT register only for real invoices — never for bank/payment rows
    unless ordinary
      if vat_data = doc.vat_data
        if !vat_data.entries.empty? && !doc.vat_month.empty?
          create_vat_journal_entry(doc, vat_data, doc_date, journal_entry.id, result)
        end
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

  # 401/411 (and other 40x/41x settlement accounts) keep counterpart subledgers.
  # VAT accounts 453x do not.
  private def settlement_account?(code : String) : Bool
    return false if code.empty? || code.starts_with?("453")
    code.starts_with?("40") || code.starts_with?("41")
  end

  private def collect_contractor_info(doc : ControlisyParser::Document) : NamedTuple(name: String, eik: String, vat: String)
    sources = [] of NamedTuple(name: String, eik: String, vat: String, weight: Int32)

    if rel = doc.rel_doc
      sources << {
        name:   (rel.contractor_name || "").strip,
        eik:    (rel.contractor_eik || "").strip,
        vat:    (rel.contractor_vat_number || "").strip,
        weight: 3,
      }
    end

    if vat_data = doc.vat_data
      sources << {
        name:   (vat_data.contractor_name || "").strip,
        eik:    "",
        vat:    (vat_data.contractor_vat_number || "").strip,
        weight: 2,
      }
    end

    doc.accountings.each do |acc|
      acc.details.each do |detail|
        name = (detail.contractor_name || "").strip
        eik = (detail.contractor_eik || "").strip
        vat = (detail.contractor_vat_number || "").strip
        next if name.empty? && eik.empty? && vat.empty?

        weight = settlement_account?(detail.account_number) ? 4 : 1
        sources << {name: name, eik: eik, vat: vat, weight: weight}
      end
    end

    name = ""
    eik = ""
    vat = ""
    sources.sort_by { |s| -s[:weight] }.each do |s|
      name = s[:name] if name.empty? && !s[:name].empty?
      eik = s[:eik] if eik.empty? && !s[:eik].empty?
      vat = s[:vat] if vat.empty? && !s[:vat].empty?
    end

    {name: name, eik: eik, vat: vat}
  end

  private def vat_variants(vat : String) : Array(String)
    v = vat.strip
    return [] of String if v.empty?
    variants = [v]
    stripped = Counterpart.strip_eu_prefix(v)
    variants << stripped unless stripped.empty?
    unless v.size > 2 && v[0].ascii_letter? && v[1].ascii_letter?
      variants << "BG#{v}"
    end
    variants.uniq
  end

  private def eik_variants(eik : String) : Array(String)
    e = eik.strip
    return [] of String if e.empty?
    variants = [e]
    stripped = Counterpart.strip_eu_prefix(e)
    variants << stripped unless stripped.empty?
    variants.uniq
  end

  private def document_number_variants(num : String) : Array(String)
    s = num.strip
    return [] of String if s.empty?
    variants = [s]
    stripped = s.lstrip('0')
    variants << stripped unless stripped.empty?
    [8, 10, 12].each do |width|
      variants << s.rjust(width, '0') if s.size < width
      unless stripped.empty? || stripped.size >= width
        variants << stripped.rjust(width, '0')
      end
    end
    variants.uniq
  end

  private def related_invoice_numbers(doc : ControlisyParser::Document) : Array(String)
    nums = [] of String
    if rel = doc.rel_doc
      if n = rel.document_number
        nums.concat(document_number_variants(n))
      end
    end
    if reason = doc.reason
      if m = reason.match(/(?:фактура|ф-ра|ф\/ра)\s*№?\s*([0-9A-Za-z\-]+)/i)
        nums.concat(document_number_variants(m[1]))
      end
    end
    nums.uniq
  end

  # Reuse the counterpart from the already imported invoice so 401/411 nets out.
  private def find_counterpart_from_related_invoice(doc : ControlisyParser::Document) : Int64?
    related_invoice_numbers(doc).each do |num|
      JournalEntryQuery.new
        .company_id(@company_id)
        .document_number(num)
        .each do |je|
          next unless je.vat_purchase_operation || je.vat_sales_operation
          if cid = je.counterpart_id
            return cid
          end
        end

      begin
        if inv = InvoiceQuery.new.company_id(@company_id).direction("sale").invoice_number(num).first?
          return inv.counterpart_id
        end
      rescue
      end
    end
    nil
  end

  private def find_counterpart_normalized(eik : String, vat_number : String, name : String = "") : Counterpart?
    normalized_eik = Counterpart.normalize_eik(eik)
    if normalized_eik.nil?
      normalized_eik = Counterpart.normalize_eik(vat_number)
    end

    # Real EIK is unique per company. 9/13 nines (non-EU / unknown) must not match.
    unless Counterpart.generic_eik?(normalized_eik)
      if found = Counterpart.find_by_unique_eik(@company_id, normalized_eik)
        return found
      end
      eik_variants(eik).each do |e|
        next if Counterpart.generic_eik?(e)
        if found = CounterpartQuery.new.company_id(@company_id).eik(e).first?
          return found
        end
        if found = CounterpartQuery.new.company_id(@company_id).vat_number(e).first?
          return found
        end
      end
    end

    unless Counterpart.generic_eik?(vat_number)
      vat_variants(vat_number).each do |v|
        next if Counterpart.generic_eik?(v)
        if found = CounterpartQuery.new.company_id(@company_id).vat_number(v).first?
          return found
        end
      end
    end

    unless name.empty?
      if found = CounterpartQuery.new.company_id(@company_id).name(name).first?
        return found
      end
    end

    nil
  end

  private def find_or_create_document_counterpart(doc : ControlisyParser::Document, result : ImportResult) : Int64?
    if cid = find_counterpart_from_related_invoice(doc)
      return cid
    end

    info = collect_contractor_info(doc)

    if existing = find_counterpart_normalized(info[:eik], info[:vat], info[:name])
      return existing.id
    end

    # Only create when the file actually names a counterpart (EIK or VAT).
    # Bank rows without contractor stay without counterpart — that is valid.
    return nil if info[:name].empty?
    return nil if info[:eik].empty? && info[:vat].empty?

    begin
      created = create_counterpart(ControlisyParser::Contractor.new(
        ca_contractor_id: "",
        name:             info[:name],
        eik:              info[:eik],
        vat_number:       info[:vat],
        inside_number:    info[:eik]
      ))
      result.contractors_created += 1
      created.id
    rescue ex
      result.warnings << "Документ #{doc.document_number}: не можа да се създаде контрагент (#{ex.message})"
      nil
    end
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

  private def build_journal_lines(doc : ControlisyParser::Document, counterpart_id : Int64? = nil) : Array(JSON::Any)
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
      attach_line_counterpart(debit_line, debit, counterpart_id)
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
      attach_line_counterpart(credit_line, credit, counterpart_id)
      if item1 = credit.account_item1
        credit_line["account_item1"] = JSON::Any.new(item1)
      end

      lines << JSON::Any.new(credit_line)
      line_num += 1
    end

    lines
  end

  private def attach_line_counterpart(
    line : Hash(String, JSON::Any),
    detail : ControlisyParser::AccountingDetail,
    counterpart_id : Int64?
  )
    if cn = detail.contractor_name
      line["counterpart_name"] = JSON::Any.new(cn) unless cn.empty?
    end
    if cv = detail.contractor_vat_number
      line["counterpart_vat"] = JSON::Any.new(cv) unless cv.empty?
    end
    if ce = detail.contractor_eik
      line["counterpart_eik"] = JSON::Any.new(ce) unless ce.empty?
    end
    if counterpart_id && settlement_account?(detail.account_number)
      line["counterpart_id"] = JSON::Any.new(counterpart_id)
    end
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

  # Controlisy bank extracts reuse one ca_docId (document type) for every row.
  # Each bank row has its own documentNumber; RelDoc.relDocumentNumber is the
  # invoice being paid and must not identify the payment (VAT does not care
  # about the bank document number).
  private def controlisy_reference_key(doc : ControlisyParser::Document, is_payment : Bool) : String
    if is_payment
      doc_no = doc.document_number.strip
      if doc_no.empty?
        doc_no = "#{doc.document_date}-#{doc.total_amount_bgn}"
      end
      "CTRL-PAY-#{doc.ca_doc_id}-#{doc_no}"
    else
      "CTRL-#{doc.ca_doc_id}"
    end
  end

  # Dedup by the unique payment/invoice key. Do not treat an invoice that
  # happens to share a number with a payment as the same document.
  private def find_existing_import(doc : ControlisyParser::Document, reference_key : String, is_payment : Bool) : JournalEntry?
    if existing = JournalEntryQuery.new
         .company_id(@company_id)
         .reference(reference_key)
         .first?
      return existing
    end

    return nil if doc.document_number.empty?

    JournalEntryQuery.new
      .company_id(@company_id)
      .document_number(doc.document_number)
      .each do |je|
        ref = je.reference || ""
        if is_payment
          next if je.vat_purchase_operation || je.vat_sales_operation
          return je if ref.starts_with?("CTRL-")
        else
          return je if ref == "CTRL-#{doc.ca_doc_id}"
        end
      end

    nil
  end

  private def patch_existing_counterpart(entry : JournalEntry, counterpart_id : Int64)
    updated_lines = entry.lines
    begin
      if parsed = JSON.parse(entry.lines).as_a?
        patched = parsed.map do |line|
          h = line.as_h
          code = h["account_code"]?.try(&.as_s?) || ""
          if settlement_account?(code)
            h["counterpart_id"] = JSON::Any.new(counterpart_id)
          end
          JSON::Any.new(h)
        end
        updated_lines = patched.to_json
      end
    rescue
    end

    SaveJournalEntry.update!(entry,
      counterpart_id: counterpart_id,
      lines: updated_lines
    )
  end

  private def contractor_dedup_key(eik : String, vat : String, name : String) : String
    normalized = Counterpart.normalize_eik(eik) || Counterpart.normalize_eik(vat)
    if normalized && !Counterpart.generic_eik?(normalized)
      "eik:#{normalized}"
    elsif !vat.empty? && !Counterpart.generic_eik?(vat)
      "vat:#{vat.strip.upcase}"
    else
      "name:#{name.strip}"
    end
  end

  private def merge_contractors(
    listed : Array(ControlisyParser::Contractor),
    extracted : Array(ControlisyParser::Contractor)
  ) : Array(ControlisyParser::Contractor)
    by_key = {} of String => ControlisyParser::Contractor
    listed.each do |c|
      by_key[contractor_dedup_key(c.eik, c.vat_number, c.name)] = c
    end
    extracted.each do |c|
      key = contractor_dedup_key(c.eik, c.vat_number, c.name)
      by_key[key] = c unless by_key.has_key?(key)
    end
    by_key.values
  end

  # Pull counterpart data from RelDoc / posting lines when <Contractors> is empty
  private def extract_contractors_from_documents(documents : Array(ControlisyParser::Document)) : Array(ControlisyParser::Contractor)
    by_key = {} of String => ControlisyParser::Contractor

    documents.each do |doc|
      candidates = [] of NamedTuple(id: String, name: String, eik: String, vat: String)

      if rel = doc.rel_doc
        candidates << {
          id:   rel.ca_rel_document_id || "",
          name: rel.contractor_name || "",
          eik:  rel.contractor_eik || "",
          vat:  rel.contractor_vat_number || "",
        }
      end

      doc.accountings.each do |acc|
        acc.details.each do |detail|
          candidates << {
            id:   "",
            name: detail.contractor_name || "",
            eik:  detail.contractor_eik || "",
            vat:  detail.contractor_vat_number || "",
          }
        end
      end

      candidates.each do |c|
        name = c[:name].strip
        next if name.empty?

        key = contractor_dedup_key(c[:eik], c[:vat], name)

        next if by_key.has_key?(key)

        by_key[key] = ControlisyParser::Contractor.new(
          ca_contractor_id: c[:id],
          name:             name,
          eik:              c[:eik],
          vat_number:       c[:vat],
          inside_number:    c[:eik]
        )
      end
    end

    by_key.values
  end

  # Detect if document is a payment (cash, card, bank) rather than an invoice.
  # Bank extracts share one ca_docId and often have no RelDoc (POS, ATM, fees,
  # internal transfers). RelDoc, when present, only links to the paid invoice.
  private def is_payment_document?(doc : ControlisyParser::Document, import_type : String = "") : Bool
    # VAT invoices are never payments
    return false if doc.vat_data

    # Whole Controlisy bank extract — every row is a bank document
    return true if import_type == "bank"

    # RelDoc + no VAT = payment against an invoice (cash rows in purchase files)
    return true if doc.rel_doc

    reason = (doc.reason || "").downcase
    payment_keywords = [
      "разплащане", "плащане", "каса", "карта", "наложен платеж",
      "payment", "транзакция", "банков", "банкова", "pos", "атм",
      "вътрешно банков",
    ]
    return true if payment_keywords.any? { |kw| reason.includes?(kw) }

    # Cash / bank / card buffer on either side (incoming or outgoing)
    doc.accountings.each do |acc|
      if debit = acc.debit_detail
        return true if debit.account_number.starts_with?("50")
      end
      if credit = acc.credit_detail
        return true if credit.account_number.starts_with?("50")
      end
    end

    false
  end
end
