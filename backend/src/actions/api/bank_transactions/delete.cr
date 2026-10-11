class Api::BankTransactions::Delete < ApiAction
  delete "/api/companies/:company_id/bank_transactions/:bank_transaction_id" do
    account_ids = BankAccountQuery.new.company_id(company_id).map(&.id)

    transaction = BankTransactionQuery.new
      .bank_account_id.in(account_ids)
      .find(bank_transaction_id)

    BankTransactionDeleter.delete!(transaction)

    json({success: true, message: "Транзакцията е изтрита"})
  rescue ex : BankTransactionDeleter::Error
    response.status_code = 400
    json({success: false, error: ex.message})
  end
end
