class Api::OpeningBalances::Subledger < ApiAction
  # Get subledger balances for counterparts or products
  get "/api/companies/:company_id/opening_balances/subledger/:type" do
    subledger_type = type # "counterpart" or "product"
    date_str = params.get?(:date) || Time.local.to_s("%Y-%m-%d")
    as_of_date = Time.parse(date_str, "%Y-%m-%d", Time::Location::UTC)

    case subledger_type
    when "counterpart"
      json(counterpart_balances(as_of_date))
    when "product"
      json(product_balances(as_of_date))
    else
      response.status_code = 400
      json({success: false, error: "Invalid subledger type. Use 'counterpart' or 'product'"})
    end
  end

  private def counterpart_balances(as_of_date : Time)
    # Get all balances with counterparts grouped by counterpart and account
    balances = OpeningBalanceQuery.new
      .company_id(company_id)
      .balance_type("counterpart")
      .date.lte(as_of_date)
      .preload_counterpart
      .preload_account

    # Group by counterpart
    grouped = {} of Int64 => {counterpart: Counterpart?, account_balances: Hash(Int64, {debit: Float64, credit: Float64, account: Account?})}

    balances.each do |b|
      next unless cid = b.counterpart_id
      grouped[cid] ||= {counterpart: b.counterpart, account_balances: {} of Int64 => {debit: Float64, credit: Float64, account: Account?}}
      aid = b.account_id
      existing = grouped[cid][:account_balances][aid]? || {debit: 0.0, credit: 0.0, account: b.account}
      grouped[cid][:account_balances][aid] = {
        debit:   existing[:debit] + b.debit,
        credit:  existing[:credit] + b.credit,
        account: existing[:account],
      }
    end

    data = grouped.map do |cid, info|
      counterpart = info[:counterpart]
      total_debit = info[:account_balances].values.sum { |v| v[:debit] }
      total_credit = info[:account_balances].values.sum { |v| v[:credit] }

      {
        counterpart_id:   cid,
        counterpart_name: counterpart.try(&.name),
        counterpart_vat:  counterpart.try(&.vat_number),
        total_debit:      total_debit,
        total_credit:     total_credit,
        net_balance:      total_debit - total_credit,
        accounts:         info[:account_balances].map do |aid, ab|
          {
            account_id:   aid,
            account_code: ab[:account].try(&.code),
            account_name: ab[:account].try(&.name),
            debit:        ab[:debit],
            credit:       ab[:credit],
            net:          ab[:debit] - ab[:credit],
          }
        end,
      }
    end

    {
      success:     true,
      as_of_date:  as_of_date.to_s("%Y-%m-%d"),
      total_debit:  data.sum { |d| d[:total_debit] },
      total_credit: data.sum { |d| d[:total_credit] },
      data:        data,
    }
  end

  private def product_balances(as_of_date : Time)
    # Get all balances with products grouped by product
    balances = OpeningBalanceQuery.new
      .company_id(company_id)
      .balance_type("product")
      .date.lte(as_of_date)
      .preload_product
      .preload_account
      .preload_warehouse

    # Group by product
    grouped = {} of Int64 => {product: Product?, total_quantity: Float64, total_value: Float64, warehouses: Hash(Int64, {quantity: Float64, value: Float64, warehouse: Warehouse?})}

    balances.each do |b|
      next unless pid = b.product_id
      grouped[pid] ||= {product: b.product, total_quantity: 0.0, total_value: 0.0, warehouses: {} of Int64 => {quantity: Float64, value: Float64, warehouse: Warehouse?}}

      wid = b.warehouse_id || 0_i64
      existing = grouped[pid][:warehouses][wid]? || {quantity: 0.0, value: 0.0, warehouse: b.warehouse}

      qty = b.quantity || 0.0
      value = b.debit - b.credit  # Net value

      grouped[pid][:warehouses][wid] = {
        quantity:  existing[:quantity] + qty,
        value:     existing[:value] + value,
        warehouse: existing[:warehouse],
      }
      grouped[pid] = grouped[pid].merge({
        total_quantity: grouped[pid][:total_quantity] + qty,
        total_value:    grouped[pid][:total_value] + value,
      })
    end

    data = grouped.map do |pid, info|
      product = info[:product]
      {
        product_id:     pid,
        product_code:   product.try(&.code),
        product_name:   product.try(&.name),
        unit:           product.try(&.measure_unit),
        total_quantity: info[:total_quantity],
        total_value:    info[:total_value],
        unit_cost:      info[:total_quantity] != 0.0 ? info[:total_value] / info[:total_quantity] : 0.0,
        warehouses:     info[:warehouses].map do |wid, wh|
          {
            warehouse_id:   wid == 0 ? nil : wid,
            warehouse_name: wh[:warehouse].try(&.name),
            quantity:       wh[:quantity],
            value:          wh[:value],
          }
        end,
      }
    end

    {
      success:        true,
      as_of_date:     as_of_date.to_s("%Y-%m-%d"),
      total_quantity: data.sum { |d| d[:total_quantity] },
      total_value:    data.sum { |d| d[:total_value] },
      data:           data,
    }
  end
end
