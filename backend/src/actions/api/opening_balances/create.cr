class Api::OpeningBalances::Create < ApiAction
  post "/api/companies/:company_id/opening_balances" do
    SaveOpeningBalance.create(params, set_company_id: company_id.to_i64) do |operation, balance|
      if balance
        full_balance = OpeningBalanceQuery.new
          .id(balance.id)
          .preload_account
          .preload_counterpart
          .preload_product
          .preload_warehouse
          .first

        json({
          success: true,
          data:    balance_json(full_balance),
        })
      else
        response.status_code = 422
        json({
          success: false,
          errors:  operation.errors.map { |key, errors| {field: key, messages: errors} },
        })
      end
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
