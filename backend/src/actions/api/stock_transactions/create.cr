class Api::StockTransactions::Create < ApiAction
  post "/api/companies/:company_id/stock_transactions" do
    response.status_code = 403
    json({
      success: false,
      error:   "Складовите движения се създават само през складови документи.",
    })
  end
end
