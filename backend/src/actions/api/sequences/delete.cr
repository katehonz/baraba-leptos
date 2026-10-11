class Api::Sequences::Delete < ApiAction
  delete "/api/companies/:company_id/sequences/:sequence_id" do
    cid = company_id.to_i64
    authorize_any!([Permission::SETTINGS_UPDATE, Permission::INVOICE_ISSUE], cid)
    DocumentNumberService.delete_interval!(cid, sequence_id.to_i64)
    json({success: true})
  rescue ex : DocumentNumberService::Error
    response.status_code = 422
    json({success: false, message: ex.message})
  end
end
