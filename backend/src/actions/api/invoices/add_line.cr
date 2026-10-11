class Api::Invoices::AddLine < ApiAction
  include Api::InvoiceParams

  post "/api/companies/:company_id/invoices/:invoice_id/lines" do
    cid = company_id.to_i64
    authorize!(Permission::INVOICE_UPDATE, cid)

    invoice = InvoiceQuery.new.company_id(cid).id(invoice_id).first
    unless invoice.draft?
      response.status_code = 422
      return json({success: false, error: "Може да се променят само чернови"})
    end

    data = nested_or_flat(params.from_json, "invoice_line")

    SaveInvoiceLine.create(
      invoice_id: invoice.id,
      description: json_str(data["description"]?) || "",
      quantity: json_f64(data["quantity"]?) || 0.0,
      unit_price: json_f64(data["unit_price"]?) || 0.0,
      vat_rate: json_f64(data["vat_rate"]?) || 20.0,
      account_id: json_i64(data["account_id"]?),
    ) do |operation, line|
      if line
        recalculate_invoice(invoice)

        json({
          success: true,
          data:    {
            id:          line.id,
            description: line.description,
            quantity:    line.quantity,
            unit_price:  line.unit_price,
            vat_rate:    line.vat_rate,
            line_total:  line.line_total,
          },
        })
      else
        response.status_code = 422
        json({
          success: false,
          errors:  operation.errors.map { |key, errors| {field: key, messages: errors} },
        })
      end
    end
  end

  private def recalculate_invoice(invoice : Invoice)
    lines = InvoiceLineQuery.new.invoice_id(invoice.id)
    subtotal = lines.sum(&.line_total)
    vat_amount = lines.sum { |l| l.line_total * l.vat_rate / 100.0 }

    SaveInvoice.update!(invoice,
      subtotal: subtotal,
      vat_amount: vat_amount,
      total_amount: subtotal + vat_amount
    )
  end
end
