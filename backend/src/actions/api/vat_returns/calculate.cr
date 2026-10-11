# Calculate VAT return from VAT journal entries
class Api::VatReturns::Calculate < ApiAction
  post "/api/companies/:company_id/vat_returns/:vat_return_id/calculate" do
    vat_return = VatReturnQuery.new
      .company_id(company_id)
      .id(vat_return_id)
      .first?

    unless vat_return
      return json({success: false, error: "VAT return not found"})
    end

    company = CompanyQuery.new.id(company_id).first?
    updated = VatReturnCalculator.calculate!(vat_return, company)
    result = updated.result_json || "{}"
    parsed = JSON.parse(result)
    vat_due = parsed["vat_due"]?.try(&.as_f?) || 0.0

    json({
      success: true,
      data:    {
        id:             updated.id,
        period_year:    updated.period_year,
        period_month:   updated.period_month,
        status:         updated.status,
        sales_count:    updated.sales_count,
        purchase_count: updated.purchase_count,
        vat_due:        vat_due,
        result_json:    result,
      },
    })
  rescue ex
    json({success: false, error: ex.message})
  end
end
