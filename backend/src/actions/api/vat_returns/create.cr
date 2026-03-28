# Create a new VAT return
class Api::VatReturns::Create < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/companies/:company_id/vat_returns" do
    cid = company_id.to_i64

    # Check if period already exists
    existing = VatReturnQuery.new
      .company_id(cid)
      .period_year(params.get(:period_year).to_i)
      .period_month(params.get(:period_month).to_i)
      .first?

    if existing
      return json({success: false, error: "VAT return for this period already exists"})
    end

    company = CompanyQuery.new.id(cid).first?

    unless company
      return json({success: false, error: "Company not found"})
    end

    vat_return = SaveVatReturn.create!(
      company_id: cid,
      period_year: params.get(:period_year).to_i,
      period_month: params.get(:period_month).to_i,
      status: "DRAFT",
      # Populate representative info from company settings
      representative_name: company.authorized_person_name || company.manager_name,
      representative_egn: company.authorized_person_egn || company.manager_egn
    )

    json({
      success: true,
      data:    {
        id:           vat_return.id,
        period_year:  vat_return.period_year,
        period_month: vat_return.period_month,
        status:       vat_return.status,
        created_at:   vat_return.created_at,
      },
    })
  rescue ex
    json({success: false, error: ex.message})
  end
end
