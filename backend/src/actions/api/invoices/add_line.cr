class Api::Invoices::AddLine < ApiAction
  post "/api/companies/:company_id/invoices/:invoice_id/lines" do
    invoice = InvoiceQuery.new.company_id(company_id).id(invoice_id).first

    SaveInvoiceLine.create(params, invoice_id: invoice.id) do |operation, line|
      if line
        # Recalculate invoice totals
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
