class Api::OpeningBalances::Index < ApiAction
  get "/api/companies/:company_id/opening_balances" do
    query = OpeningBalanceQuery.new
      .company_id(company_id)
      .preload_account
      .preload_counterpart
      .preload_product
      .preload_warehouse

    # Optional filters
    if balance_type = params.get?(:balance_type)
      query = query.balance_type(balance_type)
    end

    if account_id = params.get?(:account_id)
      query = query.account_id(account_id.to_i64)
    end

    if counterpart_id = params.get?(:counterpart_id)
      query = query.counterpart_id(counterpart_id.to_i64)
    end

    if product_id = params.get?(:product_id)
      query = query.product_id(product_id.to_i64)
    end

    if date_from = params.get?(:date_from)
      query = query.date.gte(Time.parse(date_from, "%Y-%m-%d", Time::Location::UTC))
    end

    if date_to = params.get?(:date_to)
      query = query.date.lte(Time.parse(date_to, "%Y-%m-%d", Time::Location::UTC))
    end

    balances = query.date.asc_order

    json({
      success: true,
      data:    balances.map { |b| balance_json(b) },
    })
  end

  private def balance_json(balance : OpeningBalance)
    {
      id:           balance.id,
      date:         balance.date.to_s("%Y-%m-%d"),
      debit:        balance.debit,
      credit:       balance.credit,
      quantity:     balance.quantity,
      description:  balance.description,
      balance_type: balance.balance_type,
      account:      {
        id:   balance.account.id,
        code: balance.account.code,
        name: balance.account.name,
      },
      counterpart: balance.counterpart.try { |c|
        {
          id:   c.id,
          name: c.name,
          vat:  c.vat_number,
        }
      },
      product: balance.product.try { |p|
        {
          id:   p.id,
          code: p.code,
          name: p.name,
          unit: p.measure_unit,
        }
      },
      warehouse: balance.warehouse.try { |w|
        {
          id:   w.id,
          name: w.name,
        }
      },
    }
  end
end
