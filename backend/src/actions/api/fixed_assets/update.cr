class Api::FixedAssets::Update < ApiAction
  put "/api/companies/:company_id/fixed_assets/:fixed_asset_id" do
    asset = FixedAssetQuery.new.company_id(company_id).find(fixed_asset_id)

    SaveFixedAsset.update(asset, params) do |operation, updated|
      if operation.saved?
        json({
          success: true,
          data:    {
            id:                          updated.id,
            inventory_number:            updated.inventory_number,
            name:                        updated.name,
            description:                 updated.description,
            category_id:                 updated.category_id,
            acquisition_date:            updated.acquisition_date,
            put_into_service_date:       updated.put_into_service_date,
            acquisition_cost:            updated.acquisition_cost,
            residual_value:              updated.residual_value,
            accounting_book_value:       updated.accounting_book_value,
            depreciation_method:         updated.depreciation_method,
            accounting_depreciation_rate: updated.accounting_depreciation_rate,
            tax_depreciation_rate:       updated.tax_depreciation_rate,
            status:                      updated.status,
            document_number:             updated.document_number,
            document_date:               updated.document_date,
          },
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
