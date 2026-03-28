class Api::Invoices::Update < ApiAction
  put "/api/companies/:company_id/invoices/:invoice_id" do
    invoice = InvoiceQuery.new.company_id(company_id).id(invoice_id).first

    SaveInvoice.update(invoice, params) do |operation, updated|
      if operation.saved?
        json({
          success: true,
          data:    {
            id:             updated.id,
            invoice_number: updated.invoice_number,
            issue_date:     updated.issue_date,
            due_date:       updated.due_date,
            counterpart_id: updated.counterpart_id,
            subtotal:       updated.subtotal,
            vat_amount:     updated.vat_amount,
            total_amount:   updated.total_amount,
            currency:       updated.currency,
            status:         updated.status,
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
end
