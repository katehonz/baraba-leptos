class Api::FixedAssets::Dispose < ApiAction
  post "/api/companies/:company_id/fixed_assets/:fixed_asset_id/dispose" do
    asset = FixedAssetQuery.new.company_id(company_id).find(fixed_asset_id)
    body = request.body.try(&.gets_to_end) || "{}"
    json_data = JSON.parse(body)
    date_str = json_data["disposed_at"]?.try(&.as_s?) || Time.utc.to_s("%Y-%m-%d")
    reason = json_data["disposed_reason"]?.try(&.as_s?) || "scrap"
    disposed_at = Time.parse(date_str, "%Y-%m-%d", Time::Location::UTC)

    SaveFixedAsset.update!(asset,
      disposed_at: disposed_at,
      disposed_reason: reason,
      status: "disposed"
    )

    json({success: true, data: FixedAssetQuery.new.find(asset.id).api_json})
  end
end
