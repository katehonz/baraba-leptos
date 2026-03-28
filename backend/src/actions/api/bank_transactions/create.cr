class Api::BankTransactions::Create < ApiAction
  post "/api/companies/:company_id/bank_accounts/:bank_account_id/transactions" do
    SaveBankTransaction.create(params, bank_account_id: bank_account_id.to_i64) do |operation, transaction|
      if transaction
        json({
          success: true,
          data:    {
            id:          transaction.id,
            date:        transaction.date,
            amount:      transaction.amount,
            currency:    transaction.currency,
            description: transaction.description,
          },
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
