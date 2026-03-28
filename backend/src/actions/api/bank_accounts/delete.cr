class Api::BankAccounts::Delete < ApiAction
  delete "/api/companies/:company_id/bank_accounts/:bank_account_id" do
    account = BankAccountQuery.new.company_id(company_id).find(bank_account_id)

    # Check for transactions
    tx_count = BankTransactionQuery.new.bank_account_id(account.id).select_count
    if tx_count > 0
      response.status_code = 400
      return json({success: false, error: "Сметката има #{tx_count} транзакции и не може да бъде изтрита"})
    end

    account.delete

    json({success: true, message: "Банковата сметка е изтрита"})
  end
end
