class Api::Shareholders::Delete < ApiAction
  delete "/api/companies/:company_id/shareholders/:shareholder_id" do
    shareholder = ShareholderQuery.new.company_id(company_id).find(shareholder_id)
    DeleteShareholder.delete!(shareholder)
    json({success: true, message: "Shareholder deleted"})
  end
end
