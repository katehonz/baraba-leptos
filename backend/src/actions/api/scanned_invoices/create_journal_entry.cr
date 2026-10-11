class Api::ScannedInvoices::CreateJournalEntry < ApiAction
  post "/api/companies/:company_id/scanned_invoices/:scanned_invoice_id/create_journal_entry" do
    invoice = ScannedInvoiceQuery.new.company_id(company_id).find(scanned_invoice_id)
    company = CompanyQuery.new.find(company_id)

    updated = ScannedInvoiceAccounting.book_invoice!(invoice, company)

    json({
      success: true,
      data:    {
        journal_entry_id: updated.journal_entry_id,
        message:          "Създаден е счетоводен запис ##{updated.journal_entry_id}",
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
