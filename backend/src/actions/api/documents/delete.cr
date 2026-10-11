class Api::Documents::Delete < ApiAction
  delete "/api/companies/:company_id/documents/:document_id" do
    document = DocumentQuery.new.company_id(company_id).find(document_id)
    DeleteDocument.delete!(document)
    json({success: true, message: "Document deleted successfully"})
  end
end
