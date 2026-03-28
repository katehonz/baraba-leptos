class Api::Invoices::Create < ApiAction
  post "/api/companies/:company_id/invoices" do
    SaveInvoice.create(params, company_id: company_id.to_i64) do |operation, invoice|
      if invoice
        # Use JSONB serializer
        json({
          success: true,
          data: JsonbSerializer.serialize_invoice(invoice),
        })
      else
        response.status_code = 422
        json({
          success: false,
          errors:  operation.errors.map { |key, errors| {field: key, messages: errors} },
        })
      end
    end
  end
end
