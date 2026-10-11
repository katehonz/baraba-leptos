class Api::Sequences::Create < ApiAction
  include Api::Sequences::Params

  post "/api/companies/:company_id/sequences" do
    cid = company_id.to_i64
    authorize_any!([Permission::SETTINGS_UPDATE, Permission::INVOICE_ISSUE], cid)
    company = CompanyQuery.new.find(cid)
    body = sequence_body
    next_number = read_next_number(body) || 1_i64
    seq = DocumentNumberService.create_interval!(company, read_name(body), next_number, read_document_types(body))
    json({success: true, data: DocumentNumberService.serialize(seq)})
  rescue ex : DocumentNumberService::Error
    response.status_code = 422
    json({success: false, message: ex.message})
  end
end
