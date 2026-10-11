class Api::FixedAssets::PostDepreciation < ApiAction
  post "/api/companies/:company_id/fixed_assets/post_depreciation" do
    body = request.body.try(&.gets_to_end) || "{}"
    json_data = JSON.parse(body)
    year = json_data["year"]?.try(&.as_i?) || Time.utc.year
    month = json_data["month"]?.try(&.as_i?) || Time.utc.month

    category_map = {} of Int64 => FixedAssetCategory
    FixedAssetCategoryQuery.new.company_id(company_id).each { |c| category_map[c.id] = c }
    account_map = {} of Int64 => String
    AccountQuery.new.company_id(company_id).each { |a| account_map[a.id] = a.code }

    month_start = Time.utc(year, month, 1)
    month_end = Time.utc(year, month, Time.days_in_month(year, month))

    depr_lines = [] of NamedTuple(
      asset_id: Int64,
      amount: Float64,
      tax_amount: Float64,
      book_value: Float64,
      expense_account_id: Int64,
      expense_account_code: String,
      depreciation_account_id: Int64,
      depreciation_account_code: String
    )

    FixedAssetQuery.new.company_id(company_id).each do |asset|
      next unless asset.status == "active" || asset.status == "conserved"

      already = AssetTransactionQuery.new
        .company_id(company_id.to_i64)
        .fixed_asset_id(asset.id)
        .asset_transaction_type("30")
        .in_date_range(month_start, month_end)
        .first?
      next if already

      sap = DepreciationEngine.for_asset(asset, true)
      sap_row = DepreciationEngine.year_months(sap, year).find { |r| r.month == month }
      next unless sap_row
      next if sap_row.amount <= 0.0

      dap = DepreciationEngine.for_asset(asset, false)
      tax_amount = DepreciationEngine.year_months(dap, year).find { |r| r.month == month }.try(&.amount) || 0.0

      cat = asset.category_id.try { |cid| category_map[cid]? }
      expense_id = cat.try(&.expense_account_id)
      depreciation_id = cat.try(&.depreciation_account_id)
      next if expense_id.nil? || depreciation_id.nil?

      depr_lines << {
        asset_id:                  asset.id,
        amount:                    sap_row.amount,
        tax_amount:                tax_amount,
        book_value:                sap_row.remaining,
        expense_account_id:        expense_id,
        expense_account_code:      account_map[expense_id]? || "",
        depreciation_account_id:   depreciation_id,
        depreciation_account_code: account_map[depreciation_id]? || "",
      }
    end

    if depr_lines.empty?
      response.status_code = 422
      json({success: false, message: "Няма активи за амортизиране за този период"})
    else
      je_lines = JSON.build do |j|
        j.array do
          depr_lines.each do |line|
            j.object do
              j.field "account_id", line[:expense_account_id]
              j.field "account_code", line[:expense_account_code]
              j.field "debit", line[:amount]
              j.field "credit", 0.0
            end
            j.object do
              j.field "account_id", line[:depreciation_account_id]
              j.field "account_code", line[:depreciation_account_code]
              j.field "debit", 0.0
              j.field "credit", line[:amount]
            end
          end
        end
      end

      month_str = month.to_s.rjust(2, '0')
      entry_date = month_end
      total_amount = depr_lines.sum(&.[:amount])

      SaveJournalEntry.create(
        entry_date: entry_date,
        description: "Амортизации за #{month_str}/#{year}",
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
          depr_lines.each do |line|
            asset = FixedAssetQuery.new.find(line[:asset_id])
            new_sap_acc = asset.sap_accumulated_depreciation + line[:amount]
            new_dap_acc = asset.dap_accumulated_depreciation + line[:tax_amount]
            new_book = asset.acquisition_cost - new_sap_acc
            SaveFixedAsset.update!(asset,
              sap_accumulated_depreciation: new_sap_acc,
              sap_book_value_end: new_book,
              accounting_book_value: new_book,
              dap_accumulated_depreciation: new_dap_acc,
              dap_depreciation_for_period: asset.dap_depreciation_for_period + line[:tax_amount],
              dap_tax_value_end: (asset.dap_tax_depreciable_value > 0 ? asset.dap_tax_depreciable_value : asset.acquisition_cost) - new_dap_acc
            )
            SaveAssetTransaction.create!(
              asset_transaction_id: "AT-#{asset.id}-#{year}#{month_str}-30",
              asset_transaction_type: "30",
              description: "Амортизация #{month_str}/#{year}",
              transaction_date: entry_date,
              transaction_id: entry.transaction_id,
              valuation_type: "SAP",
              acquisition_cost_on_transaction: 0.0,
              book_value_on_transaction: line[:book_value],
              transaction_amount: line[:amount],
              company_id: company_id.to_i64,
              fixed_asset_id: asset.id,
              journal_entry_id: entry.id
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
