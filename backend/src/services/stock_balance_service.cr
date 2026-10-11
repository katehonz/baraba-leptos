class StockBalanceService
  class InsufficientStock < Exception
  end

  def self.apply!(
    company : Company,
    product_id : Int64,
    warehouse_id : Int64,
    qty : Float64,
    unit_price : Float64,
    incoming : Bool,
    allow_negative : Bool
  ) : StockBalance
    bal = StockBalanceQuery.new
      .company_id(company.id)
      .product_id(product_id)
      .warehouse_id(warehouse_id)
      .for_update
      .first?

    unless bal
      bal = SaveStockBalance.create!(
        company_id: company.id,
        product_id: product_id,
        warehouse_id: warehouse_id,
        qty: 0.0,
        avg_cost: 0.0
      )
      bal = StockBalanceQuery.new.id(bal.id).for_update.first.not_nil!
    end

    old_qty = bal.qty
    old_avg = bal.avg_cost
    if incoming
      new_qty = old_qty + qty
      new_avg = if old_qty <= 0
        unit_price
      else
        Money.round((old_qty * old_avg + qty * unit_price) / new_qty)
      end
      SaveStockBalance.update!(bal, qty: new_qty, avg_cost: new_avg)
    else
      new_qty = old_qty - qty
      if new_qty < -0.0001 && !allow_negative
        raise InsufficientStock.new("Недостатъчна наличност")
      end
      SaveStockBalance.update!(bal, qty: new_qty, avg_cost: old_avg)
    end
    StockBalanceQuery.new.id(bal.id).first.not_nil!
  end

  def self.current_avg(company_id : Int64, product_id : Int64, warehouse_id : Int64) : Float64
    StockBalanceQuery.new
      .company_id(company_id)
      .product_id(product_id)
      .warehouse_id(warehouse_id)
      .first?.try(&.avg_cost) || 0.0
  end
end
