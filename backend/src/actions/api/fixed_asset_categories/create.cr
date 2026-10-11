class Api::FixedAssetCategories::Create < ApiAction
  post "/api/companies/:company_id/fixed_asset_categories" do
    SaveFixedAssetCategory.create(params, company_id: company_id.to_i64) do |operation, category|
      if category
        json({
          success: true,
          data:    {
            id:                      category.id,
            name:                    category.name,
            description:             category.description,
            cita_category:           category.cita_category,
            min_depreciation_rate:   category.min_depreciation_rate,
            max_depreciation_rate:   category.max_depreciation_rate,
            default_method:          category.default_method,
          },
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
