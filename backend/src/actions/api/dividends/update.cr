class Api::Dividends::Update < ApiAction
  put "/api/companies/:company_id/dividends/:id" do
    dividend = DividendQuery.new
      .id(id)
      .company_id(company_id)
      .first?

    if dividend
      SaveDividend.update(dividend, params) do |operation, updated|
        if updated
          # Ако е част от разпределение, проверяваме дали всички са платени
          if updated.dividend_distribution_id
            dist_id = updated.dividend_distribution_id.not_nil!
            all_dividends = DividendQuery.new.dividend_distribution_id(dist_id)
            all_paid = all_dividends.all?(&.is_paid)
            any_paid = all_dividends.any?(&.is_paid)

            # Актуализираме статуса на разпределението
            distribution = DividendDistributionQuery.new.id(dist_id).first?
            if distribution
              new_status = if all_paid
                             "paid"
                           elsif any_paid
                             "partially_paid"
                           else
                             distribution.status == "approved" ? "approved" : distribution.status
                           end

              if new_status != distribution.status
                SaveDividendDistribution.update!(distribution, status: new_status)
              end
            end
          end

          owner = updated.beneficial_owner_id ? BeneficialOwnerQuery.new.id(updated.beneficial_owner_id.not_nil!).first? : nil

          json({
            success: true,
            data:    {
              id:                   updated.id,
              beneficial_owner_id:  updated.beneficial_owner_id,
              owner_name:           owner.try(&.full_name_bg),
              gross_amount:         updated.gross_amount,
              tax_rate:             updated.tax_rate,
              tax_amount:           updated.tax_amount,
              net_amount:           updated.net_amount,
              is_paid:              updated.is_paid,
              payment_date:         updated.payment_date,
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
