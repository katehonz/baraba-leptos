class Api::FixedAssets::Delete < ApiAction
  delete "/api/companies/:company_id/fixed_assets/:fixed_asset_id" do
    asset = FixedAssetQuery.new.company_id(company_id).find(fixed_asset_id)
    DeleteFixedAsset.delete!(asset)
    json({success: true, message: "Fixed asset deleted successfully"})
  end
end
