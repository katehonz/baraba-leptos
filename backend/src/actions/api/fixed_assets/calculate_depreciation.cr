class Api::FixedAssets::CalculateDepreciation < ApiAction
  post "/api/companies/:company_id/fixed_assets/calculate_depreciation" do
    body = request.body.try(&.gets_to_end) || "{}"
    json_data = JSON.parse(body)
    year = json_data["year"]?.try(&.as_i?) || Time.utc.year
    month = json_data["month"]?.try(&.as_i?) || Time.utc.month

    categories = {} of Int64 => FixedAssetCategory
    FixedAssetCategoryQuery.new.company_id(company_id).each { |c| categories[c.id] = c }

    results = [] of NamedTuple(
      asset_id: Int64,
      inventory_number: String,
      name: String,
      acquisition_cost: Float64,
      residual_value: Float64,
      rate: Float64,
      tax_rate: Float64,
      depreciation_method: String,
      monthly_depreciation: Float64,
      tax_monthly: Float64,
      accumulated_depreciation: Float64,
      already_posted: Float64,
      conserved: Bool,
      category_name: String
    )

    FixedAssetQuery.new.company_id(company_id).each do |asset|
      next unless asset.status == "active" || asset.status == "conserved"
      sap = DepreciationEngine.for_asset(asset, true)
      dap = DepreciationEngine.for_asset(asset, false)
      sap_row = DepreciationEngine.year_months(sap, year).find { |r| r.month == month }
      dap_row = DepreciationEngine.year_months(dap, year).find { |r| r.month == month }
      next unless sap_row

      cat_name = asset.category_id.try { |id| categories[id]?.try(&.name) } || ""
      results << {
        asset_id:                 asset.id,
        inventory_number:         asset.inventory_number,
        name:                     asset.name,
        acquisition_cost:         asset.acquisition_cost,
        residual_value:           asset.residual_value || 0.0,
        rate:                     asset.accounting_depreciation_rate,
        tax_rate:                 asset.tax_depreciation_rate,
        depreciation_method:      asset.depreciation_method,
        monthly_depreciation:     sap_row.amount,
        tax_monthly:              dap_row.try(&.amount) || 0.0,
        accumulated_depreciation: sap_row.accumulated,
        already_posted:           asset.sap_accumulated_depreciation,
        conserved:                sap_row.conserved,
        category_name:            cat_name,
      }
    end

    json({
      success: true,
      data:    results,
      period:  {year: year, month: month},
    })
  end
end
