class Api::AccountingPeriods::Create < ApiAction
  post "/api/companies/:company_id/accounting_periods" do
    SaveAccountingPeriod.create(params, company_id: company_id.to_i64) do |operation, period|
      if period
        json({
          success: true,
          data:    {
            id:     period.id,
            year:   period.year,
            month:  period.month,
            status: period.status,
          },
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
