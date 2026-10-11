class Api::ScannedInvoices::Update < ApiAction
  put "/api/companies/:company_id/scanned_invoices/:scanned_invoice_id" do
    invoice = ScannedInvoiceQuery.new.company_id(company_id).find(scanned_invoice_id)

    begin
      # The preview dialog (and other JSON clients) send a flat body with JSON
      # numbers. Lucky SaveOperation looks up nested("scanned_invoice") and
      # raises MissingNestedParamError → generic 500 "Something went wrong."
      data = nested_or_flat(params.from_json, "scanned_invoice")
      counterpart_id = pick_i64(data, "counterpart_id", invoice.counterpart_id)
      if has_key?(data, "counterpart_id")
        if cp_id = counterpart_id
          unless CounterpartQuery.new.company_id(invoice.company_id).id(cp_id).first?
            response.status_code = 422
            return json({success: false, error: "Контрагентът не е от тази фирма"})
          end
        end
      end

      vies_status = if has_key?(data, "vies_status")
                     json_str(data["vies_status"]?) || invoice.vies_status
                   else
                     invoice.vies_status
                   end
      vies_validated_at = if has_key?(data, "vies_status") && (vies_status == "valid" || vies_status == "invalid")
                           Time.utc
                         else
                           invoice.vies_validated_at
                         end

      updated = SaveScannedInvoice.update!(invoice,
        vendor_name: pick_str(data, "vendor_name", invoice.vendor_name),
        vendor_vat_number: pick_str(data, "vendor_vat_number", invoice.vendor_vat_number),
        vendor_address: pick_str(data, "vendor_address", invoice.vendor_address),
        customer_name: pick_str(data, "customer_name", invoice.customer_name),
        customer_vat_number: pick_str(data, "customer_vat_number", invoice.customer_vat_number),
        customer_address: pick_str(data, "customer_address", invoice.customer_address),
        invoice_number: pick_str(data, "invoice_number", invoice.invoice_number),
        invoice_date: pick_date(data, "invoice_date", invoice.invoice_date),
        due_date: pick_date(data, "due_date", invoice.due_date),
        subtotal: pick_f64(data, "subtotal", invoice.subtotal),
        total_tax: pick_f64(data, "total_tax", invoice.total_tax),
        invoice_total: pick_f64(data, "invoice_total", invoice.invoice_total),
        notes: pick_str(data, "notes", invoice.notes),
        payment_method: pick_str(data, "payment_method", invoice.payment_method),
        counterpart_id: counterpart_id,
        vies_status: vies_status,
        vies_validation_message: pick_str(data, "vies_validation_message", invoice.vies_validation_message),
        vies_company_name: pick_str(data, "vies_company_name", invoice.vies_company_name),
        vies_company_address: pick_str(data, "vies_company_address", invoice.vies_company_address),
        vies_validated_at: vies_validated_at
      )

      json({
        success: true,
        data:    serialize(updated),
      })
    rescue ex : Time::Format::Error
      response.status_code = 422
      json({success: false, error: "Невалиден формат на дата (очакван YYYY-MM-DD)"})
    rescue ex : Avram::InvalidOperationError
      response.status_code = 422
      json({
        success: false,
        error:   "Грешка при валидация: #{ex.message}",
        errors:  ex.errors.map { |k, e| {field: k, messages: e} },
      })
    rescue ex
      Log.error { "Scanned invoice update error: #{ex.message}" }
      response.status_code = 500
      json({success: false, error: "Грешка при запазване: #{ex.message}"})
    end
  end

  private def serialize(updated_invoice : ScannedInvoice)
    {
      id:                         updated_invoice.id,
      direction:                  updated_invoice.direction,
      status:                     updated_invoice.status,
      vendor_name:                updated_invoice.vendor_name,
      vendor_vat_number:          updated_invoice.vendor_vat_number,
      vendor_address:             updated_invoice.vendor_address,
      customer_name:              updated_invoice.customer_name,
      customer_vat_number:        updated_invoice.customer_vat_number,
      customer_address:           updated_invoice.customer_address,
      invoice_number:             updated_invoice.invoice_number,
      invoice_date:               updated_invoice.invoice_date,
      due_date:                   updated_invoice.due_date,
      subtotal:                   updated_invoice.subtotal,
      total_tax:                  updated_invoice.total_tax,
      invoice_total:              updated_invoice.invoice_total,
      vies_status:                updated_invoice.vies_status,
      vies_validation_message:    updated_invoice.vies_validation_message,
      vies_company_name:          updated_invoice.vies_company_name,
      vies_company_address:       updated_invoice.vies_company_address,
      counterparty_account_id:    updated_invoice.counterparty_account_id,
      vat_account_id:             updated_invoice.vat_account_id,
      expense_revenue_account_id: updated_invoice.expense_revenue_account_id,
      requires_manual_review:     updated_invoice.requires_manual_review,
      manual_review_reason:       updated_invoice.manual_review_reason,
      notes:                      updated_invoice.notes,
      confidence:                 updated_invoice.confidence,
      counterpart_id:             updated_invoice.counterpart_id,
      vat_period:                 updated_invoice.vat_period,
      has_inventory:              updated_invoice.has_inventory,
      payment_method:             updated_invoice.payment_method,
      payment_method_raw:         updated_invoice.payment_method_raw,
      payment_journal_entry_id:   updated_invoice.payment_journal_entry_id,
    }
  end

  private def nested_or_flat(body : JSON::Any, key : String) : JSON::Any
    body[key]? || body
  end

  private def has_key?(data : JSON::Any, key : String) : Bool
    data.as_h?.try(&.has_key?(key)) || false
  end

  private def pick_str(data : JSON::Any, key : String, current : String?) : String?
    return current unless has_key?(data, key)
    json_str(data[key]?)
  end

  private def pick_f64(data : JSON::Any, key : String, current : Float64?) : Float64?
    return current unless has_key?(data, key)
    json_f64(data[key]?)
  end

  private def pick_date(data : JSON::Any, key : String, current : Time?) : Time?
    return current unless has_key?(data, key)
    val = data[key]?
    return nil if val.nil? || val.raw.nil?
    s = val.as_s?
    return nil if s.nil? || s.blank?
    slice = s.size >= 10 ? s[0, 10] : s
    Time.parse(slice, "%Y-%m-%d", Time::Location::UTC)
  end

  private def json_str(val : JSON::Any?) : String?
    return nil if val.nil? || val.raw.nil?
    s = val.as_s? || val.to_s
    s.blank? ? nil : s
  end

  private def json_i64(val : JSON::Any?) : Int64?
    return nil if val.nil? || val.raw.nil?
    val.as_i64? || val.as_i?.try(&.to_i64) || val.as_f?.try(&.to_i64) || val.as_s?.try(&.to_i64?)
  end

  private def pick_i64(data : JSON::Any, key : String, current : Int64?) : Int64?
    return current unless has_key?(data, key)
    json_i64(data[key]?)
  end

  private def json_f64(val : JSON::Any?) : Float64?
    return nil if val.nil? || val.raw.nil?
    val.as_f? ||
      val.as_i64?.try(&.to_f) ||
      val.as_i?.try(&.to_f) ||
      val.as_s?.try(&.gsub(" ", "").gsub(",", ".").to_f?)
  end
end
