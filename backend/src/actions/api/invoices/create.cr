class Api::Invoices::Create < ApiAction
  include Api::InvoiceParams

  post "/api/companies/:company_id/invoices" do
    cid = company_id.to_i64
    authorize!(Permission::INVOICE_CREATE, cid)
    company = CompanyQuery.new.find(cid)
    data = invoice_payload

    counterpart_id = json_i64(data["counterpart_id"]?)
    unless counterpart_id
      response.status_code = 422
      return json({
        success: false,
        errors:  [{field: "counterpart_id", messages: ["is required"]}],
      })
    end

    begin
      issue_date = parse_optional_date(data["issue_date"]?)
      unless issue_date
        response.status_code = 422
        return json({
          success: false,
          errors:  [{field: "issue_date", messages: ["is required"]}],
        })
      end

      validate_invoice_lines(data)

      SaveInvoice.create(
        company_id: cid,
        counterpart_id: counterpart_id,
        invoice_number: json_str(data["invoice_number"]?),
        document_type: resolved_document_type(data),
        direction: json_str(data["direction"]?) || "sale",
        issue_date: issue_date,
        due_date: parse_optional_date(data["due_date"]?),
        subtotal: json_f64(data["subtotal"]?) || 0.0,
        vat_amount: json_f64(data["vat_amount"]?) || 0.0,
        total_amount: json_f64(data["total_amount"]?) || 0.0,
        currency: json_str(data["currency"]?) || company.currency,
        exchange_rate: json_f64(data["exchange_rate"]?) || 1.0,
        payment_method: json_str(data["payment_method"]?),
        notes: json_str(data["notes"]?),
        status: json_str(data["status"]?) || "draft",
        tax_event_date: parse_optional_date(data["tax_event_date"]?),
        vat_exemption_reason: json_str(data["vat_exemption_reason"]?),
        has_inventory: json_bool(data["has_inventory"]?) || false,
        original_invoice_id: json_i64(data["original_invoice_id"]?),
        bank_account_id: json_i64(data["bank_account_id"]?),
        location_id: json_i64(data["location_id"]?),
        nap_document_type: resolved_nap_code(data),
      ) do |operation, invoice|
        if invoice
          replace_invoice_lines(invoice, data)
          invoice = InvoiceQuery.new.id(invoice.id).first.not_nil!
          json({
            success: true,
            data:    JsonbSerializer.serialize_invoice(invoice),
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
