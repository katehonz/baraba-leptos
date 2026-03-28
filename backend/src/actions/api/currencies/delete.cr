class Api::Currencies::Delete < ApiAction
  delete "/api/currencies/:currency_id" do
    currency = CurrencyQuery.new.find(currency_id)
    DeleteCurrency.delete!(currency)

    json({
      success: true,
      message: "Currency deleted successfully",
    })
  end
end
