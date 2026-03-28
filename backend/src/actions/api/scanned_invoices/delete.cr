class Api::ScannedInvoices::Delete < ApiAction
  delete "/api/companies/:company_id/scanned_invoices/:scanned_invoice_id" do
    invoice = ScannedInvoiceQuery.new.company_id(company_id).find(scanned_invoice_id)

    # Don't allow deletion if already processed
    if invoice.status == "processed" && invoice.journal_entry_id
      response.status_code = 400
      return json({success: false, error: "Не може да се изтрие осчетоводена фактура"})
    end

    invoice.delete

    json({success: true, message: "Фактурата е изтрита"})
  end
end
