class Api::Invoices::Delete < ApiAction
  delete "/api/companies/:company_id/invoices/:invoice_id" do
    invoice = InvoiceQuery.new.company_id(company_id).id(invoice_id).first
    DeleteInvoice.delete!(invoice)

    json({success: true, message: "Invoice deleted"})
  end
end
