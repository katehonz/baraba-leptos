class Api::Dividends::Create < ApiAction
  post "/api/companies/:company_id/dividends" do
    SaveDividend.create(params, company_id: company_id.to_i64) do |operation, dividend|
      if dividend
        json({
          success: true,
          data:    {
            id:            dividend.id,
            gross_amount:  dividend.gross_amount,
            tax_rate:      dividend.tax_rate,
            tax_amount:    dividend.tax_amount,
            net_amount:    dividend.net_amount,
            decision_date: dividend.decision_date,
          },
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
