class Api::DividendDistributions::Index < ApiAction
  get "/api/companies/:company_id/dividend_distributions" do
    distributions = DividendDistributionQuery.new
      .company_id(company_id)
      .created_at.desc_order

    json({
      success: true,
      data:    distributions.map { |d|
        dividends = DividendQuery.new.dividend_distribution_id(d.id)
        total_paid = dividends.select(&.is_paid).sum(&.net_amount)
        total_net = dividends.sum(&.net_amount)

        {
          id:              d.id,
          year:            d.year,
          total_amount:    d.total_amount,
          decision_date:   d.decision_date,
          decision_number: d.decision_number,
          status:          d.status,
          status_label:    DividendDistribution::STATUSES[d.status]?,
          notes:           d.notes,
          dividends_count: dividends.size,
          total_net:       total_net,
          total_paid:      total_paid,
          created_at:      d.created_at,
        }
      },
    })
  end
end
