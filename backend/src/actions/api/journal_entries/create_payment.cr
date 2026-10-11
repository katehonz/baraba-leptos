class Api::JournalEntries::CreatePayment < ApiAction
  post "/api/companies/:company_id/journal_entries/:entry_id/create_payment" do
    entry = JournalEntryQuery.new.company_id(company_id).id(entry_id).first
    company = CompanyQuery.new.find(company_id)

    method = params.get?(:method).try(&.to_s)
    if method.nil? || method.empty?
      body_str = request.body.try(&.gets_to_end) || "{}"
      json_body = JSON.parse(body_str)
      nested = json_body["journal_entry"]? || json_body
      method = nested["method"]?.try(&.as_s?)
    end

    if method.nil? || method.empty?
      response.status_code = 400
      return json({success: false, error: "Посочете начин на плащане: cash или card"})
    end

    payment = JournalEntryPayment.book!(entry, company, method.not_nil!)

    json({
      success: true,
      data:    {
        journal_entry_id:         entry.id,
        payment_journal_entry_id: payment.id,
        payment_method:           method,
        message:                  "Създадена е статия за плащане ##{payment.id}",
      },
    })
  rescue ex : JournalEntryPayment::Error
    response.status_code = 400
    json({success: false, error: ex.message})
  rescue ex : ScannedInvoiceAccounting::Error
    response.status_code = 400
    json({success: false, error: ex.message})
  rescue ex : Avram::InvalidOperationError
    response.status_code = 422
    json({success: false, error: ex.message})
  end
end
