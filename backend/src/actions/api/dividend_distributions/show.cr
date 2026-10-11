class Api::DividendDistributions::Show < ApiAction
  include Api::DividendParams

  get "/api/companies/:company_id/dividend_distributions/:id" do
    distribution = DividendDistributionQuery.new
      .id(id)
      .company_id(company_id)
      .first?

    if distribution
      dividends = DividendQuery.new.dividend_distribution_id(distribution.id)
      json({
        success: true,
        data:    serialize_distribution(distribution, dividends),
      })
    else
      response.status_code = 404
      json({success: false, message: "Not found"})
    end
  end
end
