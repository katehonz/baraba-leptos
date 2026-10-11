class InvoiceLinesError < Exception
end

# JSON body helpers for invoices (flat or nested `invoice` payload).
# Lucky SaveOperation looks up nested("invoice") and raises
# MissingNestedParamError when the frontend sends a flat JSON body.
module Api::InvoiceParams
  private def invoice_payload : JSON::Any
    nested_or_flat(params.from_json, "invoice")
  rescue
    JSON::Any.new({} of String => JSON::Any)
  end

  private def nested_or_flat(body : JSON::Any, key : String) : JSON::Any
    body[key]? || body
  end

  private def has_key?(data : JSON::Any, key : String) : Bool
    data.as_h?.try(&.has_key?(key)) || false
  end

  private def json_str(val : JSON::Any?) : String?
    return nil if val.nil? || val.raw.nil?
    s = val.as_s?
    return nil if s.nil? || s.blank?
    s.strip
  end

  private def json_i64(val : JSON::Any?) : Int64?
    return nil if val.nil? || val.raw.nil?
    val.as_i64? || val.as_i?.try(&.to_i64) || val.as_s?.try(&.to_i64?)
  end

  private def json_f64(val : JSON::Any?) : Float64?
    return nil if val.nil? || val.raw.nil?
    val.as_f? ||
      val.as_i64?.try(&.to_f) ||
      val.as_i?.try(&.to_f) ||
      val.as_s?.try(&.gsub(" ", "").gsub(",", ".").to_f?)
  end

  private def json_bool(val : JSON::Any?) : Bool?
    return nil if val.nil? || val.raw.nil?
    case val.raw
    when Bool
      val.as_bool
    when String
      s = val.as_s.downcase
      return true if s == "true" || s == "1"
      return false if s == "false" || s == "0"
      nil
    else
      nil
    end
  end

  private def parse_iso_date(value : String) : Time
    slice = value.size >= 10 ? value[0, 10] : value
    Time.parse(slice, "%Y-%m-%d", Time::Location::UTC)
  end

  private def parse_optional_date(val : JSON::Any?) : Time?
    s = json_str(val)
    return nil unless s
    parse_iso_date(s)
  end

  private def pick_str(data : JSON::Any, key : String, current : String?) : String?
    return current unless has_key?(data, key)
    json_str(data[key]?)
  end

  private def pick_i64(data : JSON::Any, key : String, current : Int64?) : Int64?
    return current unless has_key?(data, key)
    json_i64(data[key]?)
  end

  private def pick_f64(data : JSON::Any, key : String, current : Float64) : Float64
    return current unless has_key?(data, key)
    json_f64(data[key]?) || current
  end

  private def pick_bool(data : JSON::Any, key : String, current : Bool) : Bool
    return current unless has_key?(data, key)
    val = json_bool(data[key]?)
    val.nil? ? current : val
  end

  private def pick_date(data : JSON::Any, key : String, current : Time?) : Time?
    return current unless has_key?(data, key)
    parse_optional_date(data[key]?)
  end

  private def validate_invoice_lines(data : JSON::Any)
    return unless has_key?(data, "lines")
    arr = data["lines"]?.try(&.as_a?) || [] of JSON::Any
    arr.each_with_index do |ln, i|
      desc = json_str(ln["description"]?) || ""
      next if desc.blank?
      rate = json_f64(ln["vat_rate"]?) || 20.0
      exemption_id = json_i64(ln["vat_exemption_reason_id"]?)
      if rate.abs < 0.0001 && exemption_id.nil?
        raise InvoiceLinesError.new("Ред #{i + 1}: основание за неначисляване на ДДС е задължително при 0%")
      end
    end
  end

  private def replace_invoice_lines(invoice : Invoice, data : JSON::Any)
    return unless has_key?(data, "lines")
    validate_invoice_lines(data)
    InvoiceLineQuery.new.invoice_id(invoice.id).each { |l| DeleteInvoiceLine.delete!(l) }
    arr = data["lines"]?.try(&.as_a?) || [] of JSON::Any
    arr.each_with_index do |ln, i|
      desc = json_str(ln["description"]?) || ""
      next if desc.blank?
      SaveInvoiceLine.create!(
        invoice_id: invoice.id,
        description: desc,
        quantity: json_f64(ln["quantity"]?) || 1.0,
        unit_price: json_f64(ln["unit_price"]?) || 0.0,
        vat_rate: json_f64(ln["vat_rate"]?) || 20.0,
        product_id: json_i64(ln["product_id"]?),
        unit_code: json_str(ln["unit_code"]?) || "C62",
        vat_exemption_reason_id: json_i64(ln["vat_exemption_reason_id"]?),
        line_number: i + 1,
        account_id: json_i64(ln["account_id"]?)
      )
    end
    lines = InvoiceLineQuery.new.invoice_id(invoice.id)
    subtotal = lines.sum(&.line_total)
    vat = lines.sum(&.vat_amount)
    has_inv = lines.any? { |l| WarehouseDocumentPoster.inventory_line?(l) }
    reason_name = nil.as(String?)
    lines.each do |l|
      next unless l.vat_rate.abs < 0.0001
      if vid = l.vat_exemption_reason_id
        reason_name = VatExemptionReasonQuery.new.id(vid).first?.try(&.reason_name)
        break
      end
    end
    SaveInvoice.update!(invoice,
      subtotal: subtotal,
      vat_amount: vat,
      total_amount: subtotal + vat,
      has_inventory: has_inv,
      vat_exemption_reason: reason_name
    )
    InvoicePoster.rebuild_json_cache(invoice, lines)
  end

  # Проформата остава PROFORMA без код по ППЗДДС. Иначе видът за номерация
  # се извежда от кода на документа (01, 09, 81, …).
  private def resolved_document_type(data : JSON::Any) : String
    raw = json_str(data["document_type"]?) || "INVOICE"
    return "PROFORMA" if raw == "PROFORMA"
    if nap = json_str(data["nap_document_type"]?)
      return Invoice.kind_for_nap(nap) unless nap.blank?
    end
    if raw.size == 2 && raw.chars.all?(&.ascii_number?)
      return Invoice.kind_for_nap(raw)
    end
    raw
  end

  private def resolved_nap_code(data : JSON::Any) : String?
    raw = json_str(data["document_type"]?) || ""
    return nil if raw == "PROFORMA"
    nap = json_str(data["nap_document_type"]?)
    return nap unless nap.nil? || nap.blank?
    return raw if raw.size == 2 && raw.chars.all?(&.ascii_number?)
    nil
  end

  private def invoice_header_json(i : Invoice)
    {
      id:                   i.id,
      invoice_number:       i.invoice_number,
      document_type:        i.document_type,
      direction:            i.direction,
      original_invoice_id:  i.original_invoice_id,
      issue_date:           i.issue_date.to_s("%Y-%m-%d"),
      due_date:             i.due_date.try(&.to_s("%Y-%m-%d")),
      counterpart_id:       i.counterpart_id,
      subtotal:             i.subtotal,
      vat_amount:           i.vat_amount,
      total_amount:         i.total_amount,
      currency:             i.currency,
      exchange_rate:        i.exchange_rate,
      payment_method:       i.payment_method,
      notes:                i.notes,
      status:               i.status,
      tax_event_date:       i.tax_event_date.try(&.to_s("%Y-%m-%d")),
      vat_exemption_reason: i.vat_exemption_reason,
      has_inventory:        i.has_inventory,
      nap_document_type:    i.nap_document_type,
      company_id:           i.company_id,
      journal_entry_id:     i.journal_entry_id,
      bank_account_id:      i.bank_account_id,
      counterpart_name:     begin
        i.counterpart.name
      rescue
        nil
      end,
    }
  end
end
