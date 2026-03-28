class Api::BankAccounts::Create < ApiAction
  post "/api/companies/:company_id/bank_accounts" do
    SaveBankAccount.create(params, company_id: company_id.to_i64) do |operation, account|
      if account
        json({
          success: true,
          data:    {
            id:       account.id,
            name:     account.name,
            iban:     account.iban,
            currency: account.currency,
          },
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
