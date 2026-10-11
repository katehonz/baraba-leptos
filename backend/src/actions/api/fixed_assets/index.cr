class Api::FixedAssets::Index < ApiAction
  get "/api/companies/:company_id/fixed_assets" do
    assets = FixedAssetQuery.new.company_id(company_id)

    if status = params.get?("status")
      assets = assets.status(status.to_s)
    end

    json({
      success: true,
      data:    assets.map(&.api_json),
    })
  end
end
