class Api::Invoices::ReturnToDraft < ApiAction
  include Api::InvoiceParams

  post "/api/companies/:company_id/invoices/:invoice_id/return_to_draft" do
    cid = company_id.to_i64
    authorize!(Permission::INVOICE_UPDATE, cid)
    company = CompanyQuery.new.find(cid)
    invoice = InvoiceQuery.new.company_id(cid).id(invoice_id).first
    if invoice.posted? || invoice.journal_entry_id
      authorize!(Permission::ACCOUNTING_POST, cid)
    end
    result = InvoicePoster.return_to_draft!(invoice, company, current_user?.not_nil!)
    json({
      success: true,
      message: result[:message],
      data:    JsonbSerializer.serialize_invoice(result[:invoice]),
    })
  end
end
