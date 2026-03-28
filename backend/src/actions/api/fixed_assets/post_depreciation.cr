class Api::FixedAssets::PostDepreciation < ApiAction
  post "/api/companies/:company_id/fixed_assets/post_depreciation" do
    year = params.get?("year").try(&.to_i?) || Time.utc.year
    month = params.get?("month").try(&.to_i?) || Time.utc.month

    assets = FixedAssetQuery.new.company_id(company_id).status("active")
    categories = FixedAssetCategoryQuery.new.company_id(company_id)
    category_map = {} of Int64 => FixedAssetCategory
    categories.each { |c| category_map[c.id] = c }

    # Calculate depreciation for each asset
    depr_lines = [] of NamedTuple(asset_id: Int64, amount: Float64, expense_account_id: Int64, depreciation_account_id: Int64)

    assets.each do |asset|
      service_date = asset.put_into_service_date
      next if service_date.nil?

      start_year = service_date.year
      start_month = service_date.month + 1
      if start_month > 12
        start_month = 1
        start_year += 1
      end

      next if year < start_year || (year == start_year && month < start_month)

      residual = asset.residual_value || 0.0
      depreciable = asset.acquisition_cost - residual
      next if depreciable <= 0.0

      annual_depreciation = asset.acquisition_cost * (asset.accounting_depreciation_rate / 100.0)
      base_monthly = (annual_depreciation / 12.0).round(2)

      total_months = (year - start_year) * 12 + (month - start_month) + 1

      accumulated = (base_monthly * total_months).round(2)
      monthly = base_monthly

      if accumulated >= depreciable
        months_to_full = (depreciable / base_monthly).ceil.to_i
        if total_months > months_to_full
          monthly = 0.0
        elsif total_months == months_to_full
          prior = base_monthly * (total_months - 1)
          monthly = (depreciable - prior).round(2)
          monthly = 0.0 if monthly < 0.0
        end
      end

      next if monthly <= 0.0

      cat = asset.category_id.try { |cid| category_map[cid]? }
      expense_id = cat.try(&.expense_account_id)
      depreciation_id = cat.try(&.depreciation_account_id)

      next if expense_id.nil? || depreciation_id.nil?

      depr_lines << {
        asset_id:                asset.id,
        amount:                  monthly,
        expense_account_id:      expense_id,
        depreciation_account_id: depreciation_id,
      }
    end

    if depr_lines.empty?
      response.status_code = 422
      json({success: false, message: "Няма активи за амортизиране за този период"})
    else
      # Build journal entry lines JSON
      je_lines = JSON.build do |j|
        j.array do
          depr_lines.each do |line|
            j.object do
              j.field "account_id", line[:expense_account_id]
              j.field "debit", line[:amount]
              j.field "credit", 0.0
            end
            j.object do
              j.field "account_id", line[:depreciation_account_id]
              j.field "debit", 0.0
              j.field "credit", line[:amount]
            end
          end
        end
      end

      month_str = month.to_s.rjust(2, '0')
      entry_date = Time.utc(year, month, Time.days_in_month(year, month))
      description = "Амортизации за #{month_str}/#{year}"
      total_amount = depr_lines.sum(&.[:amount])

      SaveJournalEntry.create(
        entry_date: entry_date,
        description: description,
        reference: "DEPR-#{year}-#{month_str}",
        status: "posted",
        company_id: company_id.to_i64,
        lines: je_lines,
        total_amount: total_amount,
        period: month,
        period_year: year,
        journal_id: "DEPRECIATION",
        source_id: "fixed_assets",
      ) do |operation, entry|
        if entry
          # Update accumulated depreciation on each asset
          depr_lines.each do |line|
            asset = FixedAssetQuery.new.find(line[:asset_id])
            new_accumulated = asset.sap_accumulated_depreciation + line[:amount]
            SaveFixedAsset.update!(asset,
              sap_accumulated_depreciation: new_accumulated,
            )
          end

          json({
            success:          true,
            journal_entry_id: entry.id,
            total:            total_amount,
            assets_count:     depr_lines.size,
            message:          "Амортизации за #{month_str}/#{year} осчетоводени успешно",
          })
        else
          response.status_code = 422
          json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
        end
      end
    end
  end
end
