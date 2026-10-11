class PhysicalStockRebuild
  def self.rebuild!(company : Company, period_start : Time, period_end : Time) : Int32
    count = 0
    StockBalanceQuery.new.company_id(company.id).each do |bal|
      product = ProductQuery.new.id(bal.product_id).first?
      warehouse = WarehouseQuery.new.id(bal.warehouse_id).first?
      loc_code = if warehouse
        CompanyLocationQuery.new.id(warehouse.company_location_id).first?.try(&.saft_location_code)
      end
      existing = PhysicalStockQuery.new.company_id(company.id).product_id(bal.product_id).warehouse_id(bal.warehouse_id).first?
      if existing
        SavePhysicalStock.update!(existing,
          location_id: loc_code,
          product_type: product.try(&.product_type) || "30",
          uom_physical_stock: product.try(&.uom_standard) || "C62",
          unit_price_end: bal.avg_cost,
          closing_stock_quantity: bal.qty,
          closing_stock_value: Money.round(bal.qty * bal.avg_cost),
          period_start: period_start,
          period_end: period_end
        )
      else
        SavePhysicalStock.create!(
          company_id: company.id,
          product_id: bal.product_id,
          warehouse_id: bal.warehouse_id,
          location_id: loc_code,
          product_type: product.try(&.product_type) || "30",
          owner_id: "0",
          uom_physical_stock: product.try(&.uom_standard) || "C62",
          unit_price_end: bal.avg_cost,
          closing_stock_quantity: bal.qty,
          closing_stock_value: Money.round(bal.qty * bal.avg_cost),
          period_start: period_start,
          period_end: period_end
        )
      end
      count += 1
    end
    count
  end
end
