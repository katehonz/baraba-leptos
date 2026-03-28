class Api::StockTransactions::Create < ApiAction
  post "/api/companies/:company_id/stock_transactions" do
    SaveStockTransaction.create(params, company_id: company_id.to_i64) do |operation, transaction|
      if transaction
        json({
          success: true,
          data:    {
            id:               transaction.id,
            product_id:       transaction.product_id,
            warehouse_id:     transaction.warehouse_id,
            transaction_date: transaction.transaction_date,
            quantity:         transaction.quantity,
            unit_price:       transaction.unit_price,
            total_amount:     transaction.total_amount,
            is_incoming:      transaction.is_incoming,
          },
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
