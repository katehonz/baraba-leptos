class Api::BankTransactions::Delete < ApiAction
  delete "/api/companies/:company_id/bank_transactions/:bank_transaction_id" do
    # Get bank accounts for this company
    account_ids = BankAccountQuery.new.company_id(company_id).map(&.id)

    transaction = BankTransactionQuery.new
      .bank_account_id.in(account_ids)
      .find(bank_transaction_id)

    # Check if booked
    if transaction.journal_entry_id
      response.status_code = 400
      return json({success: false, error: "Осчетоводените транзакции не могат да бъдат изтрити"})
    end

    transaction.delete

    json({success: true, message: "Транзакцията е изтрита"})
  end
end
