class Api::Invoices::Ubl < ApiAction
  get "/api/companies/:company_id/invoices/:invoice_id/ubl" do
    cid = company_id.to_i64
    authorize!(Permission::INVOICE_READ, cid)
    company = CompanyQuery.new.find(cid)
    invoice = InvoiceQuery.new.company_id(cid).id(invoice_id).first
    xml = UblExportService.export(invoice, company)
    context.response.headers["Content-Disposition"] = "attachment; filename=\"invoice-#{invoice.invoice_number || invoice.id}.xml\""
    plain_text xml, content_type: "application/xml; charset=utf-8"
  end
end

