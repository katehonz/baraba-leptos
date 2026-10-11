class Api::Invoices::Index < ApiAction
  include Api::InvoiceParams

  get "/api/companies/:company_id/invoices" do
    cid = company_id.to_i64
    authorize!(Permission::INVOICE_READ, cid)

    invoices = InvoiceQuery.new.company_id(cid).preload_counterpart

    if status = params.get?("status")
      invoices = invoices.status(status.to_s)
    end

    if direction = params.get?("direction")
      invoices = invoices.direction(direction.to_s)
    end

    json({
      success: true,
      data:    invoices.map { |i| invoice_header_json(i) },
    })
  end
end
