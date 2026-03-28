class Api::OpeningBalances::Show < ApiAction
  get "/api/companies/:company_id/opening_balances/:opening_balance_id" do
    balance = OpeningBalanceQuery.new
      .id(opening_balance_id)
      .company_id(company_id)
      .preload_account
      .preload_counterpart
      .preload_product
      .preload_warehouse
      .first?

    if balance
      json({
        success: true,
        data:    balance_json(balance),
      })
    else
      response.status_code = 404
      json({success: false, error: "Opening balance not found"})
    end
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
