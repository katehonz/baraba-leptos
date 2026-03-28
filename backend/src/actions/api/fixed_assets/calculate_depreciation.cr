class Api::FixedAssets::CalculateDepreciation < ApiAction
  post "/api/companies/:company_id/fixed_assets/calculate_depreciation" do
    year = params.get?("year").try(&.to_i?) || Time.utc.year
    month = params.get?("month").try(&.to_i?) || Time.utc.month

    assets = FixedAssetQuery.new.company_id(company_id).status("active")
    categories = FixedAssetCategoryQuery.new.company_id(company_id)
    category_map = {} of Int64 => FixedAssetCategory
    categories.each { |c| category_map[c.id] = c }

    results = [] of NamedTuple(
      asset_id: Int64,
      inventory_number: String,
      name: String,
      acquisition_cost: Float64,
      residual_value: Float64,
      rate: Float64,
      depreciation_method: String,
      monthly_depreciation: Float64,
      accumulated_depreciation: Float64,
      already_posted: Float64,
      category_name: String,
    )

    assets.each do |asset|
      # Skip assets without put_into_service_date
      service_date = asset.put_into_service_date
      next if service_date.nil?

      # Depreciation starts the month AFTER put_into_service_date
      start_year = service_date.year
      start_month = service_date.month + 1
      if start_month > 12
        start_month = 1
        start_year += 1
      end

      # Check if requested period is before depreciation starts
      next if year < start_year || (year == start_year && month < start_month)

      residual = asset.residual_value || 0.0
      depreciable = asset.acquisition_cost - residual
      next if depreciable <= 0.0

      # Calculate monthly depreciation (straight_line)
      annual_depreciation = asset.acquisition_cost * (asset.accounting_depreciation_rate / 100.0)
      base_monthly = (annual_depreciation / 12.0).round(2)

      # Total months from start to requested period (inclusive)
      total_months = (year - start_year) * 12 + (month - start_month) + 1

      # Calculate accumulated depreciation up to this month
      accumulated = (base_monthly * total_months).round(2)

      # Cap at depreciable amount
      monthly = base_monthly
      if accumulated >= depreciable
        # Fully depreciated - calculate last partial month
        months_to_full = (depreciable / base_monthly).ceil.to_i
        if total_months > months_to_full
          # Already fully depreciated before this month
          monthly = 0.0
          accumulated = depreciable
        elsif total_months == months_to_full
          # This is the last month - partial amount
          prior = base_monthly * (total_months - 1)
          monthly = (depreciable - prior).round(2)
          monthly = 0.0 if monthly < 0.0
          accumulated = depreciable
        end
      end

      already_posted = asset.sap_accumulated_depreciation

      cat_name = ""
      if cid = asset.category_id
        if cat = category_map[cid]?
          cat_name = cat.name
        end
      end

      results << {
        asset_id:                asset.id,
        inventory_number:        asset.inventory_number,
        name:                    asset.name,
        acquisition_cost:        asset.acquisition_cost,
        residual_value:          residual,
        rate:                    asset.accounting_depreciation_rate,
        depreciation_method:     asset.depreciation_method,
        monthly_depreciation:    monthly,
        accumulated_depreciation: accumulated,
        already_posted:          already_posted,
        category_name:           cat_name,
      }
    end

    json({
      success: true,
      data:    results,
      period:  {year: year, month: month},
    })
  end
end
