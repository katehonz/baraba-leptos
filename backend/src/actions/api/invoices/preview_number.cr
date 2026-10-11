class Api::Invoices::PreviewNumber < ApiAction
  get "/api/companies/:company_id/sequences/preview" do
    cid = company_id.to_i64
    authorize!(Permission::INVOICE_READ, cid)
    company = CompanyQuery.new.find(cid)
    DocumentNumberService.initialize_for_company(company)
    doc_type = params.get?("document_type").try(&.to_s) || "INVOICE"
    seq = DocumentNumberService.sequence_for(cid, "invoice_sale", doc_type)
    unless seq
      response.status_code = 404
      return json({success: false, error: "Няма верига"})
    end
    json({success: true, data: {preview: DocumentNumberService.preview(seq), sequence_id: seq.id}})
  end
end
