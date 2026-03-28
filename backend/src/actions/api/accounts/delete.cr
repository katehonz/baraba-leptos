class Api::Accounts::Delete < ApiAction
  delete "/api/companies/:company_id/accounts/:account_id" do
    account = AccountQuery.new.company_id(company_id).id(account_id).first?

    unless account
      response.status_code = 404
      return json({success: false, error: "Сметката не е намерена"})
    end

    DeleteAccount.delete!(account)
    json({success: true, message: "Сметката е изтрита"})
  rescue ex
    response.status_code = 422
    json({success: false, error: ex.message || "Грешка при изтриване"})
  end
end
