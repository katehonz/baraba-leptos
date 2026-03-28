# Import VAT journal entries from accounting journal entries
# This creates VatJournalEntry records from JournalEntry records for a given period
class Api::VatReturns::ImportFromJournal < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/companies/:company_id/vat_returns/:vat_return_id/import_from_journal" do
    cid = company_id.to_i64
    vrid = vat_return_id.to_i64

    vat_return = VatReturnQuery.new
      .company_id(cid)
      .id(vrid)
      .first?

    unless vat_return
      return json({success: false, error: "VAT return not found"})
    end

    # Build period string like "2025-01"
    period = "#{vat_return.period_year}-#{vat_return.period_month.to_s.rjust(2, '0')}"

    # Get all journal entries for this period that have VAT operations
    entries = JournalEntryQuery.new
      .company_id(cid)
      .vat_period(period)

    # Delete all auto-imported entries first (idempotent reimport)
    # This prevents duplicates regardless of race conditions or repeated clicks
    # Only deletes entries that were auto-imported (have source_journal_entry_id set)
    deleted_count = 0
    VatJournalEntryQuery.new
      .vat_return_id(vrid)
      .source_journal_entry_id.is_not_nil
      .each do |existing|
        existing.delete
        deleted_count += 1
      end

    purchase_count = 0
    sales_count = 0

    entries.each do |entry|
      # Get counterpart info from counterpart_id if available
      counterpart_name = ""
      counterpart_vat = ""
      goods_description = ""

      if cp_id = entry.counterpart_id
        if counterpart = CounterpartQuery.new.id(cp_id).first?
          counterpart_name = counterpart.name || ""
          counterpart_vat = counterpart.vat_number || ""
        end
      end

      # Fallback: Parse counterpart info from lines if not found from counterpart_id
      if counterpart_name.empty?
        if lines_json = entry.lines
          begin
            lines = JSON.parse(lines_json)
            if lines.as_a?
              lines.as_a.each do |line|
                if cp_name = line["counterpart_name"]?.try(&.as_s?)
                  counterpart_name = cp_name if counterpart_name.empty?
                end
                if cp_vat = line["counterpart_vat"]?.try(&.as_s?)
                  counterpart_vat = cp_vat if counterpart_vat.empty?
                end
                if desc = line["description"]?.try(&.as_s?)
                  goods_description = desc if goods_description.empty?
                end
              end
            end
          rescue
            # Ignore JSON parse errors
          end
        end
      end

      # Use entry description if no goods description from lines
      if goods_description.empty? && entry.description
        goods_description = entry.description.not_nil!
      end

      vat_amount = entry.total_vat_amount || 0.0
      base_amount = ((entry.total_amount || 0.0) - vat_amount).round(2)
      doc_date = entry.document_date || entry.entry_date

      # Import purchases
      if vat_op = entry.vat_purchase_operation
          unless vat_op.empty?
            purchase_count += 1

            # Extract code and label from format "01|PURCHASE_CREDIT"
            parts = vat_op.split("|")
            vat_code = parts.first? || "01"
            op_label = parts.last? || ""

            # Map vat_purchase_operation to document_type and amounts
            doc_type, base_no_cr, base_full, vat_full, base_partial, vat_partial, vop_base, vop_vat, base_9, vat_9 = map_purchase_operation(vat_code, op_label, base_amount, vat_amount)

            # Ensure doc_type is a valid NAP code
            doc_type = sanitize_doc_type(doc_type, "purchase")

            SaveVatJournalEntry.create!(
              company_id: cid,
              vat_return_id: vrid,
              row_number: purchase_count,
              document_type: doc_type,
              document_number: entry.document_number || "",
              document_date: doc_date,
              counterpart_vat: counterpart_vat.empty? ? nil : counterpart_vat,
              counterpart_name: counterpart_name.empty? ? nil : counterpart_name,
              goods_description: goods_description.empty? ? nil : goods_description[0, 30],
              entry_type: "purchase",
              # Amounts based on operation type
              base_no_credit: base_no_cr,
              base_full_credit: base_full,
              vat_full_credit: vat_full,
              base_partial_credit: base_partial,
              vat_partial_credit: vat_partial,
              vop_base: vop_base,
              vop_vat: vop_vat,
              base_9: base_9,
              vat_9: vat_9,
              source_journal_entry_id: entry.id,
              notes: "Imported from journal entry ##{entry.id}"
            )
          end
        end

      # Import sales
      if vat_op = entry.vat_sales_operation
          unless vat_op.empty?
            sales_count += 1

            # Extract code and label from format "01|SALES_COL_22"
            # Note: first part may be a long description from Controlisy like "5 – Доставка..."
            parts = vat_op.split("|")
            vat_code = parts.first? || "01"
            op_label = parts.last? || ""

            # Map vat_sales_operation to document_type and amounts
            doc_type, base_20, vat_20, base_9, vat_9, base_0, base_vod, base_0_articles, base_services_21, base_exempt = map_sales_operation(vat_code, op_label, base_amount, vat_amount)

            # Ensure doc_type is a valid NAP code (2 digits), extract leading number if needed
            doc_type = sanitize_doc_type(doc_type, "sales")

            SaveVatJournalEntry.create!(
              company_id: cid,
              vat_return_id: vrid,
              row_number: sales_count,
              document_type: doc_type,
              document_number: entry.document_number || "",
              document_date: doc_date,
              counterpart_vat: counterpart_vat.empty? ? nil : counterpart_vat,
              counterpart_name: counterpart_name.empty? ? nil : counterpart_name,
              goods_description: goods_description.empty? ? nil : goods_description[0, 30],
              entry_type: "sales",
              # Amounts based on operation type
              total_base: base_amount,
              total_vat: vat_amount,
              sales_base_20: base_20,
              sales_vat_20: vat_20,
              sales_base_9: base_9,
              sales_vat_9: vat_9,
              sales_base_0_chapter3: base_0,
              sales_base_vod: base_vod,
              sales_base_0_articles: base_0_articles,
              sales_base_services_21: base_services_21,
              sales_base_exempt: base_exempt,
              source_journal_entry_id: entry.id,
              notes: "Imported from journal entry ##{entry.id}"
            )
          end
        end
    end

    # Renumber all entries after reimport
    ["purchase", "sales"].each do |etype|
      row = 1
      VatJournalEntryQuery.new
        .vat_return_id(vrid)
        .entry_type(etype)
        .id.asc_order
        .each do |e|
          if e.row_number != row
            SaveVatJournalEntry.update!(e, row_number: row)
          end
          row += 1
        end
    end

    json({
      success: true,
      data:    {
        purchase_count: purchase_count,
        sales_count:    sales_count,
        deleted_count:  deleted_count,
        message:        "Импортирани #{purchase_count} покупки и #{sales_count} продажби (изтрити #{deleted_count} стари)",
      },
    })
  rescue ex
    json({success: false, error: ex.message})
  end

  # Map purchase VAT operation code and label to document type and amount columns
  # Returns: {doc_type, base_no_credit, base_full_credit, vat_full_credit, base_partial, vat_partial, vop_base, vop_vat, base_9, vat_9}
  # Labels from Controlisy: PURCHASE_CREDIT, PURCHASE_PARTIAL, PURCHASE_NO_CREDIT,
  #   PURCHASE_VOP, PURCHASE_NO_RIGHT, ICA
  private def map_purchase_operation(vat_code : String, op_label : String, base : Float64, vat : Float64)
    # Use label as primary dispatch (like sales)
    case op_label
    when "PURCHASE_CREDIT"
      return {vat_code, 0.0, base, vat, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0}
    when "PURCHASE_PARTIAL", "PURCHASE_COL_12"
      return {vat_code, 0.0, 0.0, 0.0, base, vat, 0.0, 0.0, 0.0, 0.0}
    when "PURCHASE_NO_CREDIT", "PURCHASE_COL_15"
      return {vat_code, base, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0}
    when "PURCHASE_NO_RIGHT"
      return {vat_code, base, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0}
    when "PURCHASE_VOP", "ICA"
      return {vat_code, 0.0, 0.0, 0.0, 0.0, 0.0, base, vat, 0.0, 0.0}
    when "PURCHASE_COL_14"
      return {vat_code, 0.0, base, vat, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0}
    end

    # Fallback: use numeric vat_code
    case vat_code
    when "01" # Standard purchase 20% - full credit
      {"01", 0.0, base, vat, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0}
    when "02" # Purchase 9%
      {"01", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, base, vat}
    when "03" # Purchase without VAT (no credit)
      {"01", base, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0}
    when "05" # Trilateral intermediary acquisition (VOP-like)
      {"09", 0.0, 0.0, 0.0, 0.0, 0.0, base, vat, 0.0, 0.0}
    when "06" # No credit right (v2)
      {"01", base, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0}
    when "09" # Intra-EU purchase (VOP)
      {"09", 0.0, 0.0, 0.0, 0.0, 0.0, base, vat, 0.0, 0.0}
    else
      {"01", 0.0, base, vat, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0}
    end
  end

  # Map sales VAT operation code to document type and amount columns
  # Returns: {doc_type, base_20, vat_20, base_9, vat_9, base_0, base_vod, base_0_articles, base_services_21, base_exempt}
  # Labels from Controlisy: SALES_20, SALES_COL_17, EXPORT_ZERO, ICS, SALES_COL_21-24, SALES_VOP
  private def map_sales_operation(vat_code : String, op_label : String, base : Float64, vat : Float64)
    # First check the operation label (from frontend or Controlisy: "SALES_COL_22", etc.)
    case op_label
    when "SALES_20", "DOMESTIC_SALE"
      return {vat_code, base, vat, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0}
    when "SALES_COL_17"
      return {vat_code, 0.0, 0.0, base, vat, 0.0, 0.0, 0.0, 0.0, 0.0}
    when "EXPORT_ZERO"
      return {vat_code, 0.0, 0.0, 0.0, 0.0, base, 0.0, 0.0, 0.0, 0.0}
    when "ICS"
      return {vat_code, 0.0, 0.0, 0.0, 0.0, 0.0, base, 0.0, 0.0, 0.0}
    when "SALES_COL_21"
      return {vat_code, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, base, 0.0, 0.0}
    when "SALES_COL_22"
      return {vat_code, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, base, 0.0}
    when "SALES_COL_23"
      # Art.69 para 2 - for now maps to same as SALES_COL_22 (services)
      return {vat_code, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, base, 0.0}
    when "SALES_COL_24"
      return {vat_code, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, base}
    when "SALES_VOP"
      # Protocol VOP/Art.82 in sales register - base goes to total, amounts handled by VatJournalEntry directly
      return {vat_code, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0}
    end

    # Fallback: use numeric vat_code (from Controlisy imports)
    case vat_code
    when "01" # Standard sale 20%
      {"01", base, vat, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0}
    when "02" # Sale 9%
      {"01", 0.0, 0.0, base, vat, 0.0, 0.0, 0.0, 0.0, 0.0}
    when "03" # Sale 0% chapter 3
      {"01", 0.0, 0.0, 0.0, 0.0, base, 0.0, 0.0, 0.0, 0.0}
    when "04" # Intra-EU delivery (VOD)
      {"01", 0.0, 0.0, 0.0, 0.0, 0.0, base, 0.0, 0.0, 0.0}
    when "05" # Services Art.21 para 2
      {"01", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, base, 0.0}
    when "06" # Art.69 para 2
      {"01", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, base, 0.0}
    when "07" # Distance sales
      {"01", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, base}
    when "08" # Exempt
      {"01", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, base}
    when "10" # Art.140, 146, 173
      {"01", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, base, 0.0, 0.0}
    when "11" # Intra-EU sale (VOD) - legacy code
      {"01", 0.0, 0.0, 0.0, 0.0, 0.0, base, 0.0, 0.0, 0.0}
    when "12" # Exempt - legacy code
      {"01", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, base}
    else
      {"01", base, vat, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0}
    end
  end

  # Sanitize document type to ensure it's a valid NAP code
  # Handles cases where Controlisy stores full description like "5 – Доставка на услуги..."
  private def sanitize_doc_type(doc_type : String, entry_type : String) : String
    dt = doc_type.strip
    # Already valid 2-digit code
    return dt if dt.size <= 2 && dt.chars.all?(&.ascii_number?)

    # Try to extract leading number (e.g., "5 – Description..." -> "05")
    if match = dt.match(/^(\d+)/)
      code = match[1].rjust(2, '0')
      valid_types = if entry_type == "purchase"
        VatJournalEntry::PURCHASE_DOC_TYPES.keys
      else
        VatJournalEntry::SALES_DOC_TYPES.keys
      end
      return code if valid_types.includes?(code)
    end

    # Default to "01" (Invoice)
    "01"
  end
end
