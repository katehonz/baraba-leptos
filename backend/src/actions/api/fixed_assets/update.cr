class Api::FixedAssets::Update < ApiAction
  put "/api/companies/:company_id/fixed_assets/:fixed_asset_id" do
    asset = FixedAssetQuery.new.company_id(company_id).find(fixed_asset_id)

    SaveFixedAsset.update(asset, params) do |operation, updated|
      if operation.saved?
        json({success: true, data: updated.api_json})
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
