class Api::OpeningBalances::TrialBalance < ApiAction
  get "/api/companies/:company_id/opening_balances/trial_balance" do
    date_str = params.get?(:date) || Time.local.to_s("%Y-%m-%d")
    as_of_date = Time.parse(date_str, "%Y-%m-%d", Time::Location::UTC)

    # Get all accounts with their opening balances
    accounts = AccountQuery.new.company_id(company_id).is_active(true)

    trial_balance_data = accounts.map do |account|
      # Sum opening balances up to this date
      opening_balances = OpeningBalanceQuery.new
        .company_id(company_id)
        .account_id(account.id)
        .date.lte(as_of_date)

      total_debit = 0.0
      total_credit = 0.0
      opening_balances.each do |ob|
        total_debit += ob.debit
        total_credit += ob.credit
      end

      # Get journal entry movements (if posted entries exist)
      # This would require summing JournalLine debits/credits for this account
      # For now, we show only opening balances

      net_balance = total_debit - total_credit

      {
        account_id:    account.id,
        account_code:  account.code,
        account_name:  account.name,
        account_type:  account.account_type,
        opening_debit:  total_debit,
        opening_credit: total_credit,
        net_balance:    net_balance,
      }
    end

    # Filter out accounts with zero balance
    non_zero = trial_balance_data.reject { |tb| tb[:opening_debit] == 0.0 && tb[:opening_credit] == 0.0 }

    # Calculate totals
    total_debit = non_zero.sum { |tb| tb[:opening_debit] }
    total_credit = non_zero.sum { |tb| tb[:opening_credit] }

    json({
      success:      true,
      as_of_date:   date_str,
      total_debit:  total_debit,
      total_credit: total_credit,
      is_balanced:  (total_debit - total_credit).abs < 0.01,
      data:         non_zero,
    })
  end
end
