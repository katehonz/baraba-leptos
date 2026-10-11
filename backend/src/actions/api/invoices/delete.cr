class Api::Invoices::Delete < ApiAction
  delete "/api/companies/:company_id/invoices/:invoice_id" do
    cid = company_id.to_i64
    authorize!(Permission::INVOICE_DELETE, cid)

    invoice = InvoiceQuery.new.company_id(cid).id(invoice_id).first
    unless invoice.draft?
      response.status_code = 422
      return json({success: false, error: "Може да се изтриват само чернови"})
    end

    DeleteInvoice.delete!(invoice)

    json({success: true, message: "Invoice deleted"})
  end
end
