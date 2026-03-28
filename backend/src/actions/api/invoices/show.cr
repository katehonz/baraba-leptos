class Api::Invoices::Show < ApiAction
  get "/api/companies/:company_id/invoices/:invoice_id" do
    invoice = InvoiceQuery.new.company_id(company_id).id(invoice_id).first
    lines = InvoiceLineQuery.new.invoice_id(invoice.id)

    json({
      success: true,
      data:    {
        id:             invoice.id,
        invoice_number: invoice.invoice_number,
        issue_date:     invoice.issue_date,
        due_date:       invoice.due_date,
        counterpart_id: invoice.counterpart_id,
        subtotal:       invoice.subtotal,
        vat_amount:     invoice.vat_amount,
        total_amount:   invoice.total_amount,
        currency:       invoice.currency,
        status:         invoice.status,
        notes:          invoice.notes,
        lines:          lines.map { |l|
          {
            id:          l.id,
            description: l.description,
            quantity:    l.quantity,
            unit_price:  l.unit_price,
            vat_rate:    l.vat_rate,
            line_total:  l.line_total,
          }
        },
      },
    })
  end
end
