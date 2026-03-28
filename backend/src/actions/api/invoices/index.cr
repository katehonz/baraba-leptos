class Api::Invoices::Index < ApiAction
  get "/api/companies/:company_id/invoices" do
    invoices = InvoiceQuery.new.company_id(company_id)

    # Filter by status if provided
    if status = params.get?("status")
      invoices = invoices.status(status.to_s)
    end

    json({
      success: true,
      data:    invoices.map { |i| invoice_json(i) },
    })
  end

  private def invoice_json(i : Invoice)
    {
      id:             i.id,
      invoice_number: i.invoice_number,
      issue_date:     i.issue_date,
      due_date:       i.due_date,
      counterpart_id: i.counterpart_id,
      subtotal:       i.subtotal,
      vat_amount:     i.vat_amount,
      total_amount:   i.total_amount,
      currency:       i.currency,
      status:         i.status,
      notes:          i.notes,
    }
  end
end
