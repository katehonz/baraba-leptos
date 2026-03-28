class Api::BankAccounts::Update < ApiAction
  put "/api/companies/:company_id/bank_accounts/:bank_account_id" do
    account = BankAccountQuery.new.company_id(company_id).find(bank_account_id)

    SaveBankAccount.update(account, params) do |operation, updated|
      if operation.saved?
        json({
          success: true,
          data:    {
            id:            updated.id,
            name:          updated.name,
            iban:          updated.iban,
            bic:           updated.bic,
            currency:      updated.currency,
            bank_name:     updated.bank_name,
            gl_account_id: updated.gl_account_id,
          },
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
