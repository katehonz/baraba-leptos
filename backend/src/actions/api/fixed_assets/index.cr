class Api::FixedAssets::Index < ApiAction
  get "/api/companies/:company_id/fixed_assets" do
    assets = FixedAssetQuery.new.company_id(company_id)

    # Filter by status if provided
    if status = params.get?("status")
      assets = assets.status(status.to_s)
    end

    json({
      success: true,
      data:    assets.map { |a| asset_json(a) },
    })
  end

  private def asset_json(a : FixedAsset)
    {
      id:                          a.id,
      inventory_number:            a.inventory_number,
      name:                        a.name,
      description:                 a.description,
      category_id:                 a.category_id,
      acquisition_date:            a.acquisition_date,
      put_into_service_date:       a.put_into_service_date,
      acquisition_cost:            a.acquisition_cost,
      residual_value:              a.residual_value,
      accounting_book_value:       a.accounting_book_value,
      depreciation_method:         a.depreciation_method,
      accounting_depreciation_rate: a.accounting_depreciation_rate,
      tax_depreciation_rate:       a.tax_depreciation_rate,
      status:                      a.status,
    }
  end
end
