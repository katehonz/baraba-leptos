class Api::Invoices::Issue < ApiAction
  post "/api/companies/:company_id/invoices/:invoice_id/issue" do
    cid = company_id.to_i64
    authorize!(Permission::INVOICE_ISSUE, cid)
    company = CompanyQuery.new.find(cid)
    invoice = InvoiceQuery.new.company_id(cid).id(invoice_id).first
    updated = InvoicePoster.issue!(invoice, company)
    json({success: true, data: JsonbSerializer.serialize_invoice(updated)})
  end
end
