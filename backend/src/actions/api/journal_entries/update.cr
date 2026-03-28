class Api::JournalEntries::Update < ApiAction
  put "/api/companies/:company_id/journal_entries/:entry_id" do
    begin
      entry = JournalEntryQuery.new.company_id(company_id).id(entry_id).first?

      unless entry
        response.status_code = 404
        return json({success: false, error: "Записът не е намерен"})
      end

      if entry.status == "posted"
        response.status_code = 422
        return json({success: false, error: "Cannot modify posted journal entry"})
      end

      # Parse JSON body
      body_str = request.body.try(&.gets_to_end) || "{}"
      json_body = JSON.parse(body_str)

      # Extract fields from JSON
      entry_date_str = json_body["entry_date"]?.try(&.as_s?) || entry.entry_date.to_s("%Y-%m-%d")
      entry_date = Time.parse(entry_date_str, "%Y-%m-%d", Time::Location::UTC)
      document_date_str = json_body["document_date"]?.try(&.as_s?)
      document_date = if dds = document_date_str
                        Time.parse(dds, "%Y-%m-%d", Time::Location::UTC)
                      else
                        entry.document_date
                      end
      description = json_body["description"]?.try(&.as_s?) || entry.description
      reference = json_body["reference"]?.try(&.as_s?)
      status = json_body["status"]?.try(&.as_s?) || entry.status
      document_number = json_body["document_number"]?.try(&.as_s?)
      vat_period = json_body["vat_period"]?.try(&.as_s?)
      vat_purchase_operation = json_body["vat_purchase_operation"]?.try(&.as_s?)
      vat_sales_operation = json_body["vat_sales_operation"]?.try(&.as_s?)
      lines = json_body["lines"]?.try(&.as_s?) || entry.lines
      counterpart_id = json_body["counterpart_id"]?.try(&.as_i64?)

      SaveJournalEntry.update!(entry,
        entry_date: entry_date,
        document_date: document_date,
        description: description,
        reference: reference,
        status: status,
        document_number: document_number,
        vat_period: vat_period,
        vat_purchase_operation: vat_purchase_operation,
        vat_sales_operation: vat_sales_operation,
        lines: lines,
        counterpart_id: counterpart_id
      )

      # Cascade update to linked VAT journal entries
      update_linked_vat_entries(entry, entry_date, counterpart_id)

      json({
        success: true,
        data:    {
          id:              entry.id,
          entry_date:      entry.entry_date,
          description:     entry.description,
          reference:       entry.reference,
          status:          entry.status,
          vat_period:      entry.vat_period,
          document_number: entry.document_number,
        },
      })
    rescue ex : Avram::InvalidOperationError
      response.status_code = 422
      json({
        success: false,
        error:   "Грешка при валидация: #{ex.message}",
      })
    rescue ex
      Log.error { "Journal entry update error: #{ex.message}" }
      response.status_code = 500
      json({success: false, error: "Грешка при обновяване: #{ex.message}"})
    end
  end

  # Update linked VAT journal entries when the journal entry changes
  private def update_linked_vat_entries(entry : JournalEntry, entry_date : Time, counterpart_id : Int64?)
    linked = VatJournalEntryQuery.new
      .source_journal_entry_id(entry.id)

    # Get counterpart info
    counterpart_name = ""
    counterpart_vat = ""
    if cp_id = counterpart_id
      if counterpart = CounterpartQuery.new.id(cp_id).first?
        counterpart_name = counterpart.name || ""
        counterpart_vat = counterpart.vat_number || ""
      end
    end

    vat_amount = entry.total_vat_amount || 0.0
    base_amount = ((entry.total_amount || 0.0) - vat_amount).round(2)
    doc_date = entry.document_date || entry_date
    goods_description = (entry.description || "")[0, 30]

    linked.each do |vat_entry|
      if vat_entry.entry_type == "purchase"
        # Map purchase operation
        if vat_op = entry.vat_purchase_operation
          unless vat_op.empty?
            op_label = vat_op.split("|").last? || ""
            amounts = map_purchase_amounts(op_label, base_amount, vat_amount)

            SaveVatJournalEntry.update!(vat_entry,
              document_number: entry.document_number || vat_entry.document_number,
              document_date: doc_date,
              counterpart_vat: counterpart_vat.empty? ? vat_entry.counterpart_vat : counterpart_vat,
              counterpart_name: counterpart_name.empty? ? vat_entry.counterpart_name : counterpart_name,
              goods_description: goods_description.empty? ? vat_entry.goods_description : goods_description,
              base_no_credit: amounts[:base_no_credit],
              base_full_credit: amounts[:base_full_credit],
              vat_full_credit: amounts[:vat_full_credit],
              base_partial_credit: amounts[:base_partial_credit],
              vat_partial_credit: amounts[:vat_partial_credit],
              vop_base: amounts[:vop_base],
              vop_vat: amounts[:vop_vat]
            )
          end
        end
      elsif vat_entry.entry_type == "sales"
        # Map sales operation
        if vat_op = entry.vat_sales_operation
          unless vat_op.empty?
            op_label = vat_op.split("|").last? || ""
            raw_doc_type = vat_op.split("|").first? || "01"
            # Extract valid doc type: handle long descriptions like "5 – Доставка..."
            doc_type_code = if raw_doc_type.size <= 2 && raw_doc_type.chars.all?(&.ascii_number?)
                              raw_doc_type
                            elsif match = raw_doc_type.match(/^(\d+)/)
                              match[1].rjust(2, '0')
                            else
                              "01"
                            end
            amounts = map_sales_amounts(op_label, base_amount, vat_amount)

            SaveVatJournalEntry.update!(vat_entry,
              document_type: doc_type_code,
              document_number: entry.document_number || vat_entry.document_number,
              document_date: doc_date,
              counterpart_vat: counterpart_vat.empty? ? vat_entry.counterpart_vat : counterpart_vat,
              counterpart_name: counterpart_name.empty? ? vat_entry.counterpart_name : counterpart_name,
              goods_description: goods_description.empty? ? vat_entry.goods_description : goods_description,
              total_base: amounts[:total_base],
              total_vat: amounts[:total_vat],
              sales_base_20: amounts[:sales_base_20],
              sales_vat_20: amounts[:sales_vat_20],
              sales_base_vop: amounts[:sales_base_vop],
              sales_vat_vop: amounts[:sales_vat_vop],
              sales_base_9: amounts[:sales_base_9],
              sales_vat_9: amounts[:sales_vat_9],
              sales_base_0_chapter3: amounts[:sales_base_0_chapter3],
              sales_base_vod: amounts[:sales_base_vod],
              sales_base_0_articles: amounts[:sales_base_0_articles],
              sales_base_services_21: amounts[:sales_base_services_21],
              sales_base_69_2: amounts[:sales_base_69_2],
              sales_base_69_2_eu: amounts[:sales_base_69_2_eu],
              sales_base_exempt: amounts[:sales_base_exempt],
              sales_vat_personal: amounts[:sales_vat_personal],
              sales_base_vop_9: amounts[:sales_base_vop_9]
            )
          end
        end
      end
    end
  rescue ex
    Log.warn { "Failed to update linked VAT entries for journal #{entry.id}: #{ex.message}" }
  end

  # Map sales operation label to amount columns
  private def map_sales_amounts(op_label : String, base : Float64, vat : Float64)
    # Reset all to zero, then set appropriate columns
    result = {
      total_base:           0.0,
      total_vat:            0.0,
      sales_base_20:        0.0,
      sales_vat_20:         0.0,
      sales_base_vop:       0.0,
      sales_vat_vop:        0.0,
      sales_base_9:         0.0,
      sales_vat_9:          0.0,
      sales_base_0_chapter3: 0.0,
      sales_base_vod:       0.0,
      sales_base_0_articles: 0.0,
      sales_base_services_21: 0.0,
      sales_base_69_2:      0.0,
      sales_base_69_2_eu:   0.0,
      sales_base_exempt:    0.0,
      sales_vat_personal:   0.0,
      sales_base_vop_9:     0.0,
    }

    case op_label
    when "SALES_20", "DOMESTIC_SALE"
      result = result.merge({total_base: base, total_vat: vat, sales_base_20: base, sales_vat_20: vat})
    when "SALES_COL_17"
      result = result.merge({total_base: base, total_vat: vat, sales_base_9: base, sales_vat_9: vat})
    when "EXPORT_ZERO"
      result = result.merge({total_base: base, sales_base_0_chapter3: base})
    when "ICS"
      result = result.merge({total_base: base, sales_base_vod: base})
    when "SALES_COL_21"
      result = result.merge({total_base: base, sales_base_0_articles: base})
    when "SALES_COL_22"
      result = result.merge({total_base: base, sales_base_services_21: base})
    when "SALES_COL_23"
      result = result.merge({total_base: base, sales_base_69_2: base})
    when "SALES_COL_24"
      result = result.merge({total_base: base, sales_base_exempt: base})
    when "SALES_VOP"
      result = result.merge({total_base: base, total_vat: vat, sales_base_vop: base, sales_vat_vop: vat})
    else
      # Default: standard 20%
      result = result.merge({total_base: base, total_vat: vat, sales_base_20: base, sales_vat_20: vat})
    end

    result
  end

  # Map purchase operation label to amount columns
  private def map_purchase_amounts(op_label : String, base : Float64, vat : Float64)
    result = {
      base_no_credit:      0.0,
      base_full_credit:    0.0,
      vat_full_credit:     0.0,
      base_partial_credit: 0.0,
      vat_partial_credit:  0.0,
      vop_base:            0.0,
      vop_vat:             0.0,
    }

    case op_label
    when "PURCHASE_CREDIT"
      result = result.merge({base_full_credit: base, vat_full_credit: vat})
    when "PURCHASE_NO_CREDIT"
      result = result.merge({base_no_credit: base})
    when "PURCHASE_NO_RIGHT"
      result = result.merge({base_no_credit: base})
    when "PURCHASE_PARTIAL", "PURCHASE_COL_12"
      result = result.merge({base_partial_credit: base, vat_partial_credit: vat})
    when "PURCHASE_VOP"
      result = result.merge({vop_base: base, vop_vat: vat})
    when "ICA"
      result = result.merge({base_full_credit: base, vat_full_credit: vat, vop_base: base, vop_vat: vat})
    when "PURCHASE_COL_14"
      result = result.merge({base_full_credit: base, vat_full_credit: vat})
    when "PURCHASE_COL_15"
      result = result.merge({base_no_credit: base})
    else
      # Default: full credit
      result = result.merge({base_full_credit: base, vat_full_credit: vat})
    end

    result
  end
end
