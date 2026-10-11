class Api::WarehouseDocuments::Index < ApiAction
  get "/api/companies/:company_id/warehouse_documents" do
    cid = company_id.to_i64
    authorize!(Permission::WAREHOUSE_READ, cid)
    docs = WarehouseDocumentQuery.new.company_id(cid)
    if status = params.get?("status")
      docs = docs.status(status.to_s)
    end
    json({
      success: true,
      data:    docs.map { |d|
        {
          id:              d.id,
          document_type:   d.document_type,
          document_number: d.document_number,
          issue_date:      d.issue_date.to_s("%Y-%m-%d"),
          status:          d.status,
          from_location_id: d.from_location_id,
          to_location_id:   d.to_location_id,
          invoice_id:      d.invoice_id,
          movement_type:   d.movement_type,
        }
      },
    })
  end
end
