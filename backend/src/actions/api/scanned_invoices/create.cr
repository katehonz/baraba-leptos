class Api::ScannedInvoices::Create < ApiAction
  post "/api/companies/:company_id/scanned_invoices" do
    SaveScannedInvoice.create(params, company_id: company_id.to_i64) do |operation, invoice|
      if invoice
        json({
          success: true,
          data:    {
            id:        invoice.id,
            direction: invoice.direction,
            status:    invoice.status,
          },
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
