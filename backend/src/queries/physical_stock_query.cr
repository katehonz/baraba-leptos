class PhysicalStockQuery < PhysicalStock::BaseQuery
  def by_company(company_id : Int64)
    company_id(company_id)
  end

  def by_warehouse(warehouse_id : Int64)
    warehouse_id(warehouse_id)
  end

  def by_product(product_id : Int64)
    product_id(product_id)
  end

  def by_owner(owner_id : String)
    owner_id(owner_id)
  end

  def by_product_type(product_type : String)
    product_type(product_type)
  end

  def in_period(start_date : Time, end_date : Time)
    period_start.gte(start_date).period_end.lte(end_date)
  end

  def with_stock
    closing_stock_quantity.gt(0.0)
  end
end
