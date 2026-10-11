class Api::BankAccounts::Update < ApiAction
  include Api::BankAccountParams

  put "/api/companies/:company_id/bank_accounts/:bank_account_id" do
    account = BankAccountQuery.new.company_id(company_id).find(bank_account_id)
    data = bank_account_payload

    SaveBankAccount.update(account,
      name: json_str(data["name"]?) || account.name,
      iban: json_str(data["iban"]?) || account.iban,
      bic: has_key?(data, "bic") ? json_str(data["bic"]?) : account.bic,
      currency: json_str(data["currency"]?) || account.currency,
      bank_name: has_key?(data, "bank_name") ? json_str(data["bank_name"]?) : account.bank_name,
      gl_account_id: has_key?(data, "gl_account_id") ? json_i64(data["gl_account_id"]?) : account.gl_account_id,
      buffer_account_id: has_key?(data, "buffer_account_id") ? json_i64(data["buffer_account_id"]?) : account.buffer_account_id,
    ) do |operation, updated|
      if operation.saved?
        json({
          success: true,
          data:    serialize_bank_account(updated),
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
