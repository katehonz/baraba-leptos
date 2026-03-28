class Api::ExchangeRates::Delete < ApiAction
  delete "/api/exchange_rates/:exchange_rate_id" do
    exchange_rate = ExchangeRateQuery.new.find(exchange_rate_id)
    DeleteExchangeRate.delete!(exchange_rate)

    json({
      success: true,
      message: "Exchange rate deleted successfully",
    })
  end
end
