class Api::DividendDistributions::Update < ApiAction
  put "/api/companies/:company_id/dividend_distributions/:id" do
    distribution = DividendDistributionQuery.new
      .id(id)
      .company_id(company_id)
      .first?

    if distribution
      SaveDividendDistribution.update(distribution, params) do |operation, updated|
        if updated
          json({
            success: true,
            data:    {
              id:              updated.id,
              year:            updated.year,
              total_amount:    updated.total_amount,
              decision_date:   updated.decision_date,
              decision_number: updated.decision_number,
              status:          updated.status,
              status_label:    DividendDistribution::STATUSES[updated.status]?,
              notes:           updated.notes,
            },
          })
        else
          response.status_code = 422
          json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
        end
      end
    else
      response.status_code = 404
      json({success: false, message: "Not found"})
    end
  end
end
