class Api::DividendDistributions::Create < ApiAction
  post "/api/companies/:company_id/dividend_distributions" do
    SaveDividendDistribution.create(params, company_id: company_id.to_i64) do |operation, distribution|
      if distribution
        # Взимаме всички активни действителни собственици с процент на собственост
        beneficial_owners = BeneficialOwnerQuery.new
          .company_id(company_id)
          .is_active(true)
          .to_a
          .select { |bo| bo.ownership_percentage && bo.ownership_percentage.not_nil! > 0 }

        # Изчисляваме дивиденти за всеки собственик
        calculated = distribution.calculate_dividends(beneficial_owners)

        # Създаваме записи за дивиденти
        created_dividends = [] of Dividend
        calculated.each do |calc|
          SaveDividend.create!(
            company_id: company_id.to_i64,
            dividend_distribution_id: distribution.id,
            beneficial_owner_id: calc[:beneficial_owner_id],
            gross_amount: calc[:gross_amount],
            tax_rate: calc[:tax_rate],
            tax_amount: calc[:tax_amount],
            net_amount: calc[:net_amount],
            decision_date: distribution.decision_date,
            is_paid: false
          )
        end

        # Зареждаме създадените дивиденти
        dividends = DividendQuery.new.dividend_distribution_id(distribution.id)

        json({
          success: true,
          data:    {
            id:              distribution.id,
            year:            distribution.year,
            total_amount:    distribution.total_amount,
            decision_date:   distribution.decision_date,
            decision_number: distribution.decision_number,
            status:          distribution.status,
            notes:           distribution.notes,
            dividends:       dividends.map { |d|
              owner = d.beneficial_owner_id ? BeneficialOwnerQuery.new.id(d.beneficial_owner_id.not_nil!).first? : nil
              {
                id:                   d.id,
                beneficial_owner_id:  d.beneficial_owner_id,
                owner_name:           owner.try(&.full_name_bg),
                ownership_percentage: owner.try(&.ownership_percentage),
                gross_amount:         d.gross_amount,
                tax_rate:             d.tax_rate,
                tax_amount:           d.tax_amount,
                net_amount:           d.net_amount,
                is_paid:              d.is_paid,
              }
            },
          },
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
