class Api::ExchangeRates::Index < ApiAction
  get "/api/exchange_rates" do
    exchange_rates = ExchangeRateQuery.new.is_active(true)

    # Filter by date if provided
    if date_str = params.get?("date")
      # Parse date and filter
    end

    json({
      success: true,
      data:    exchange_rates.map { |e| exchange_rate_json(e) },
    })
  end

  private def exchange_rate_json(e : ExchangeRate)
    {
      id:               e.id,
      from_currency_id: e.from_currency_id,
      to_currency_id:   e.to_currency_id,
      rate:             e.rate,
      reverse_rate:     e.reverse_rate,
      valid_date:       e.valid_date,
      rate_source:      e.rate_source,
      is_active:        e.is_active,
      notes:            e.notes,
    }
  end
end
