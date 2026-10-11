class Api::StockBalances::Index < ApiAction
  get "/api/companies/:company_id/stock_balances" do
    cid = company_id.to_i64
    authorize!(Permission::WAREHOUSE_READ, cid)
    bals = StockBalanceQuery.new.company_id(cid)
    json({
      success: true,
      data:    bals.map { |b|
        product = ProductQuery.new.id(b.product_id).first?
        warehouse = WarehouseQuery.new.id(b.warehouse_id).first?
        {
          id:            b.id,
          product_id:    b.product_id,
          product_code:  product.try(&.code),
          product_name:  product.try(&.name),
          warehouse_id:  b.warehouse_id,
          warehouse_name: warehouse.try(&.name),
          qty:           b.qty,
          avg_cost:      b.avg_cost,
        }
      },
    })
  end
end
