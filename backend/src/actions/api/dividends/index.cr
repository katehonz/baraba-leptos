class Api::Dividends::Index < ApiAction
  get "/api/companies/:company_id/dividends" do
    dividends = DividendQuery.new.company_id(company_id)

    json({
      success: true,
      data:    dividends.map { |d|
        owner = d.beneficial_owner_id ? BeneficialOwnerQuery.new.id(d.beneficial_owner_id.not_nil!).first? : nil

        {
          id:                       d.id,
          shareholder_id:           d.shareholder_id,
          beneficial_owner_id:      d.beneficial_owner_id,
          dividend_distribution_id: d.dividend_distribution_id,
          owner_name:               owner.try(&.full_name_bg),
          ownership_percentage:     owner.try(&.ownership_percentage),
          gross_amount:             d.gross_amount,
          tax_rate:                 d.tax_rate,
          tax_amount:               d.tax_amount,
          net_amount:               d.net_amount,
          decision_date:            d.decision_date,
          payment_date:             d.payment_date,
          is_paid:                  d.is_paid,
        }
      },
    })
  end
end
