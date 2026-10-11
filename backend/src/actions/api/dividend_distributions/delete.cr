class Api::DividendDistributions::Delete < ApiAction
  delete "/api/companies/:company_id/dividend_distributions/:id" do
    distribution = DividendDistributionQuery.new
      .id(id)
      .company_id(company_id)
      .first?

    if distribution
      if distribution.status != "draft"
        response.status_code = 422
        return json({
          success: false,
          message: "Може да се изтрие само чернова. Първо върнете разпределението в чернова.",
        })
      end

      any_paid = DividendQuery.new.dividend_distribution_id(distribution.id).any?(&.is_paid)
      if any_paid
        response.status_code = 422
        return json({
          success: false,
          message: "Не може да се изтрие разпределение с изплатени дивиденти. Първо върнете в чернова.",
        })
      end

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
