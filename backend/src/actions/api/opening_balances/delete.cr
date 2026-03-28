class Api::OpeningBalances::Delete < ApiAction
  delete "/api/companies/:company_id/opening_balances/:opening_balance_id" do
    balance = OpeningBalanceQuery.new
      .id(opening_balance_id)
      .company_id(company_id)
      .first?

    if balance.nil?
      response.status_code = 404
      return json({success: false, error: "Opening balance not found"})
    end

    balance.delete

    json({success: true, message: "Opening balance deleted"})
  end
end
