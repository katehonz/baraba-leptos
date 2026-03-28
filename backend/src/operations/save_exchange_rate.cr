class SaveExchangeRate < ExchangeRate::SaveOperation
  permit_columns rate, reverse_rate, valid_date, rate_source, is_active, notes,
    from_currency_id, to_currency_id, created_by_id

  before_save do
    validate_required rate, valid_date
    validate_inclusion_of rate_source, in: ["ecb", "manual", "fixed"]

    # Calculate reverse rate if not provided
    if reverse_rate.value.nil? && rate.value
      r = rate.value.not_nil!
      if r > 0
        reverse_rate.value = 1.0 / r
      end
    end
  end
end
