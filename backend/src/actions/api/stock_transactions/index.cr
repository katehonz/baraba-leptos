class Api::StockTransactions::Index < ApiAction
  get "/api/companies/:company_id/stock_transactions" do
    transactions = StockTransactionQuery.new.company_id(company_id)

    # Filter by warehouse if provided
    if warehouse_id = params.get?("warehouse_id")
      transactions = transactions.warehouse_id(warehouse_id.to_i64)
    end

    # Filter by product if provided
    if product_id = params.get?("product_id")
      transactions = transactions.product_id(product_id.to_i64)
    end

    json({
      success: true,
      data:    transactions.map { |t| {
        id:               t.id,
        product_id:       t.product_id,
        warehouse_id:     t.warehouse_id,
        transaction_date: t.transaction_date,
        quantity:         t.quantity,
        unit_price:       t.unit_price,
        total_amount:     t.total_amount,
        is_incoming:      t.is_incoming,
        movement_type:    t.movement_type,
        document_number:  t.document_number,
      } },
    })
  end
end
