class Api::Invoices::Cancel < ApiAction
  include Api::InvoiceParams

  post "/api/companies/:company_id/invoices/:invoice_id/cancel" do
    cid = company_id.to_i64
    authorize!(Permission::INVOICE_CANCEL, cid)
    invoice = InvoiceQuery.new.company_id(cid).id(invoice_id).first
    data = invoice_payload
    updated = InvoicePoster.cancel!(invoice, json_str(data["cancellation_reason"]?))
    json({success: true, data: JsonbSerializer.serialize_invoice(updated)})
  end
end
