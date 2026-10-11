class Api::WarehouseDocuments::Show < ApiAction
  get "/api/companies/:company_id/warehouse_documents/:doc_id" do
    cid = company_id.to_i64
    authorize!(Permission::WAREHOUSE_READ, cid)
    doc = WarehouseDocumentQuery.new.company_id(cid).id(doc_id).first
    lines = WarehouseDocumentLineQuery.new.warehouse_document_id(doc.id)
    json({
      success: true,
      data:    {
        id:              doc.id,
        document_type:   doc.document_type,
        document_number: doc.document_number,
        issue_date:      doc.issue_date.to_s("%Y-%m-%d"),
        status:          doc.status,
        from_location_id: doc.from_location_id,
        to_location_id:   doc.to_location_id,
        notes:           doc.notes,
        invoice_id:      doc.invoice_id,
        lines:           lines.map { |l|
          {
            id:          l.id,
            product_id:  l.product_id,
            quantity:    l.quantity,
            unit_code:   l.unit_code,
            unit_price:  l.unit_price,
            line_total:  l.line_total,
            line_number: l.line_number,
          }
        },
      },
    })
  end
end
