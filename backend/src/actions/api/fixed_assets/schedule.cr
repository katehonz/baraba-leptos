class Api::FixedAssets::Schedule < ApiAction
  get "/api/companies/:company_id/fixed_assets/:fixed_asset_id/schedule" do
    asset = FixedAssetQuery.new.company_id(company_id).find(fixed_asset_id)
    year = params.get?("year").try(&.to_i?) || Time.utc.year

    sap_params = DepreciationEngine.for_asset(asset, true)
    dap_params = DepreciationEngine.for_asset(asset, false)
    sap_rows = DepreciationEngine.schedule(sap_params, year, 12)
    dap_rows = DepreciationEngine.schedule(dap_params, year, 12)

    json({
      success: true,
      data:    {
        asset: asset.api_json,
        sap:   sap_rows.map { |r| month_json(r) },
        dap:   dap_rows.map { |r| month_json(r) },
      },
    })
  end

  private def month_json(r : DepreciationEngine::MonthRow)
    {
      year:          r.year,
      month:         r.month,
      cost:          r.cost,
      useful_months: r.useful_months,
      monthly_rate:  r.monthly_rate,
      amount:        r.amount,
      accumulated:   r.accumulated,
      remaining:     r.remaining,
      conserved:     r.conserved,
      note:          r.note,
    }
  end
end
