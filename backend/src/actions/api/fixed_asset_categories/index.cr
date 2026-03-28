class Api::FixedAssetCategories::Index < ApiAction
  get "/api/companies/:company_id/fixed_asset_categories" do
    categories = FixedAssetCategoryQuery.new.company_id(company_id)

    json({
      success: true,
      data:    categories.map { |c| category_json(c) },
    })
  end

  private def category_json(c : FixedAssetCategory)
    {
      id:                      c.id,
      name:                    c.name,
      description:             c.description,
      cita_category:           c.cita_category,
      min_depreciation_rate:   c.min_depreciation_rate,
      max_depreciation_rate:   c.max_depreciation_rate,
      default_method:          c.default_method,
      asset_account_id:        c.asset_account_id,
      depreciation_account_id: c.depreciation_account_id,
      expense_account_id:      c.expense_account_id,
    }
  end
end
