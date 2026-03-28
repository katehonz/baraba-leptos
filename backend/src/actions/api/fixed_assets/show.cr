class Api::FixedAssets::Show < ApiAction
  get "/api/companies/:company_id/fixed_assets/:fixed_asset_id" do
    asset = FixedAssetQuery.new.company_id(company_id).find(fixed_asset_id)

    json({
      success: true,
      data:    {
        id:                          asset.id,
        inventory_number:            asset.inventory_number,
        name:                        asset.name,
        description:                 asset.description,
        category_id:                 asset.category_id,
        acquisition_date:            asset.acquisition_date,
        put_into_service_date:       asset.put_into_service_date,
        acquisition_cost:            asset.acquisition_cost,
        residual_value:              asset.residual_value,
        accounting_book_value:       asset.accounting_book_value,
        depreciation_method:         asset.depreciation_method,
        accounting_depreciation_rate: asset.accounting_depreciation_rate,
        tax_depreciation_rate:       asset.tax_depreciation_rate,
        status:                      asset.status,
        document_number:             asset.document_number,
        document_date:               asset.document_date,
      },
    })
  end
end
