class SaveStockBalance < StockBalance::SaveOperation
  permit_columns company_id, product_id, warehouse_id, qty, avg_cost
end
