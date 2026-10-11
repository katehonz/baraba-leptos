class Api::WarehouseDocuments::Post < ApiAction
  post "/api/companies/:company_id/warehouse_documents/:doc_id/post" do
    cid = company_id.to_i64
    authorize!(Permission::WAREHOUSE_POST, cid)
    company = CompanyQuery.new.find(cid)
    doc = WarehouseDocumentQuery.new.company_id(cid).id(doc_id).first
    updated = WarehouseDocumentPoster.post!(doc, company, company.allow_negative_stock?)
    json({success: true, data: {id: updated.id, status: updated.status, document_number: updated.document_number}})
  end
end
