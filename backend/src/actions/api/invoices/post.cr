class Api::Invoices::Post < ApiAction
  post "/api/companies/:company_id/invoices/:invoice_id/post" do
    cid = company_id.to_i64
    authorize!(Permission::ACCOUNTING_POST, cid)
    company = CompanyQuery.new.find(cid)
    invoice = InvoiceQuery.new.company_id(cid).id(invoice_id).first
    updated = InvoicePoster.post!(invoice, company, current_user?.not_nil!)
    json({success: true, data: JsonbSerializer.serialize_invoice(updated)})
  end
end
