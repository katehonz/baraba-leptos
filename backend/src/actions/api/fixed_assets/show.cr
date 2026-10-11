class Api::FixedAssets::Show < ApiAction
  get "/api/companies/:company_id/fixed_assets/:fixed_asset_id" do
    asset = FixedAssetQuery.new.company_id(company_id).find(fixed_asset_id)
    json({success: true, data: asset.api_json})
  end
end
