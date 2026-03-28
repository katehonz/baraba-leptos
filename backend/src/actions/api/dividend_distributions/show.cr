class Api::DividendDistributions::Show < ApiAction
  get "/api/companies/:company_id/dividend_distributions/:id" do
    distribution = DividendDistributionQuery.new
      .id(id)
      .company_id(company_id)
      .first?

    if distribution
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
          status_label:    DividendDistribution::STATUSES[distribution.status]?,
          notes:           distribution.notes,
          created_at:      distribution.created_at,
          dividends:       dividends.map { |d|
            owner = d.beneficial_owner_id ? BeneficialOwnerQuery.new.id(d.beneficial_owner_id.not_nil!).first? : nil
            {
              id:                   d.id,
              beneficial_owner_id:  d.beneficial_owner_id,
              owner_name:           owner.try(&.full_name_bg),
              owner_egn:            owner.try(&.egn),
              ownership_percentage: owner.try(&.ownership_percentage),
              gross_amount:         d.gross_amount,
              tax_rate:             d.tax_rate,
              tax_amount:           d.tax_amount,
              net_amount:           d.net_amount,
              is_paid:              d.is_paid,
              payment_date:         d.payment_date,
            }
          },
        },
      })
    else
      response.status_code = 404
      json({success: false, message: "Not found"})
    end
  end
end
