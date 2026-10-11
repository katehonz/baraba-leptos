class Api::Sequences::Update < ApiAction
  include Api::Sequences::Params

  put "/api/companies/:company_id/sequences/:sequence_id" do
    cid = company_id.to_i64
    authorize_any!([Permission::SETTINGS_UPDATE, Permission::INVOICE_ISSUE], cid)
    body = sequence_body
    next_number = read_next_number(body)
    unless next_number
      response.status_code = 422
      return json({success: false, message: "Въведете следващ номер"})
    end
    seq = DocumentNumberService.update_interval!(cid, sequence_id.to_i64, read_name(body), next_number.not_nil!)
    json({success: true, data: DocumentNumberService.serialize(seq)})
  rescue ex : DocumentNumberService::Error
    response.status_code = 422
    json({success: false, message: ex.message})
  end
end
