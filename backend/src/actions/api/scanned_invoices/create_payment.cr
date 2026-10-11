# Creates a SEPARATE payment journal entry (cash or card buffer) for a scanned invoice.
class Api::ScannedInvoices::CreatePayment < ApiAction
  post "/api/companies/:company_id/scanned_invoices/:scanned_invoice_id/create_payment" do
    invoice = ScannedInvoiceQuery.new.company_id(company_id).find(scanned_invoice_id)
    company = CompanyQuery.new.find(company_id)

    method = params.get?(:method) || params.get?("method").try(&.to_s)
    if method.nil? || method.empty?
      body_str = request.body.try(&.gets_to_end) || "{}"
      json_body = JSON.parse(body_str)
      method = json_body["method"]?.try(&.as_s?)
    end

    if method.nil? || method.empty?
      response.status_code = 400
      return json({success: false, error: "Посочете начин на плащане: cash или card"})
    end

    updated = ScannedInvoiceAccounting.book_payment!(invoice, company, method.not_nil!)

    json({
      success: true,
      data:    {
        journal_entry_id:         updated.journal_entry_id,
        payment_journal_entry_id: updated.payment_journal_entry_id,
        payment_method:           updated.payment_method,
        message:                  "Създадена е статия за плащане ##{updated.payment_journal_entry_id}",
      },
    })
  rescue ex : ScannedInvoiceAccounting::Error
    response.status_code = 400
    json({success: false, error: ex.message})
  rescue ex : Avram::InvalidOperationError
    response.status_code = 422
    json({success: false, error: ex.message})
  end
end
