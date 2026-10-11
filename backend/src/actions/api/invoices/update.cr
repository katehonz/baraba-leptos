class Api::Invoices::Update < ApiAction
  include Api::InvoiceParams

  put "/api/companies/:company_id/invoices/:invoice_id" do
    cid = company_id.to_i64
    authorize!(Permission::INVOICE_UPDATE, cid)

    invoice = InvoiceQuery.new.company_id(cid).id(invoice_id).first
    unless invoice.draft?
      response.status_code = 422
      return json({success: false, error: "Може да се променят само чернови"})
    end

    data = invoice_payload
    touch_kind = has_key?(data, "document_type") || has_key?(data, "nap_document_type")

    begin
      validate_invoice_lines(data)
      SaveInvoice.update(invoice,
        counterpart_id: pick_i64(data, "counterpart_id", invoice.counterpart_id) || invoice.counterpart_id,
        invoice_number: pick_str(data, "invoice_number", invoice.invoice_number),
        document_type: touch_kind ? resolved_document_type(data) : invoice.document_type,
        direction: pick_str(data, "direction", invoice.direction) || invoice.direction,
        issue_date: pick_date(data, "issue_date", invoice.issue_date) || invoice.issue_date,
        due_date: pick_date(data, "due_date", invoice.due_date),
        subtotal: pick_f64(data, "subtotal", invoice.subtotal),
        vat_amount: pick_f64(data, "vat_amount", invoice.vat_amount),
        total_amount: pick_f64(data, "total_amount", invoice.total_amount),
        currency: pick_str(data, "currency", invoice.currency) || invoice.currency,
        exchange_rate: pick_f64(data, "exchange_rate", invoice.exchange_rate),
        payment_method: pick_str(data, "payment_method", invoice.payment_method),
        notes: pick_str(data, "notes", invoice.notes),
        status: "draft",
        tax_event_date: pick_date(data, "tax_event_date", invoice.tax_event_date),
        vat_exemption_reason: pick_str(data, "vat_exemption_reason", invoice.vat_exemption_reason),
        has_inventory: pick_bool(data, "has_inventory", invoice.has_inventory),
        original_invoice_id: pick_i64(data, "original_invoice_id", invoice.original_invoice_id),
        bank_account_id: pick_i64(data, "bank_account_id", invoice.bank_account_id),
        location_id: pick_i64(data, "location_id", invoice.location_id),
        nap_document_type: touch_kind ? resolved_nap_code(data) : invoice.nap_document_type,
      ) do |operation, updated|
        if operation.saved?
          replace_invoice_lines(updated, data)
          updated = InvoiceQuery.new.id(updated.id).first.not_nil!
          json({
            success: true,
            data:    invoice_header_json(updated),
          })
        else
          response.status_code = 422
          json({
            success: false,
            errors:  operation.errors.map { |key, errors| {field: key, messages: errors} },
          })
        end
      end
    rescue ex : InvoiceLinesError
      response.status_code = 422
      json({success: false, error: ex.message})
    rescue ex : Time::Format::Error
      response.status_code = 422
      json({success: false, error: "Невалиден формат на дата (очакван YYYY-MM-DD)"})
    end
  end
end
