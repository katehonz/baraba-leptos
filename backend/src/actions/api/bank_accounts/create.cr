class Api::BankAccounts::Create < ApiAction
  include Api::BankAccountParams

  post "/api/companies/:company_id/bank_accounts" do
    data = bank_account_payload

    SaveBankAccount.create(
      company_id: company_id.to_i64,
      name: json_str(data["name"]?) || "",
      iban: json_str(data["iban"]?) || "",
      bic: json_str(data["bic"]?),
      currency: json_str(data["currency"]?) || "EUR",
      bank_name: json_str(data["bank_name"]?),
      gl_account_id: json_i64(data["gl_account_id"]?),
      buffer_account_id: json_i64(data["buffer_account_id"]?),
    ) do |operation, account|
      if account
        json({
          success: true,
          data:    serialize_bank_account(account),
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
