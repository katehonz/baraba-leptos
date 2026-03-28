class Api::DividendDistributions::Delete < ApiAction
  delete "/api/companies/:company_id/dividend_distributions/:id" do
    distribution = DividendDistributionQuery.new
      .id(id)
      .company_id(company_id)
      .first?

    if distribution
      # Изтриваме свързаните дивиденти първо
      DividendQuery.new.dividend_distribution_id(distribution.id).each do |dividend|
        DeleteDividend.delete!(dividend)
      end

      # Изтриваме разпределението
      distribution.delete

      json({success: true})
    else
      response.status_code = 404
      json({success: false, message: "Not found"})
    end
  end
end
