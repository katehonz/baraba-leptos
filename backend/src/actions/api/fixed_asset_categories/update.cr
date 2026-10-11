class Api::FixedAssetCategories::Update < ApiAction
  put "/api/companies/:company_id/fixed_asset_categories/:fixed_asset_category_id" do
    category = FixedAssetCategoryQuery.new.company_id(company_id).find(fixed_asset_category_id)

    SaveFixedAssetCategory.update(category, params) do |operation, updated|
      if operation.saved?
        json({
          success: true,
          data:    {
            id:                    updated.id,
            name:                  updated.name,
            description:           updated.description,
            cita_category:         updated.cita_category,
            min_depreciation_rate: updated.min_depreciation_rate,
            max_depreciation_rate: updated.max_depreciation_rate,
            default_method:        updated.default_method,
            asset_account_id:      updated.asset_account_id,
            depreciation_account_id: updated.depreciation_account_id,
            expense_account_id:    updated.expense_account_id,
          },
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
