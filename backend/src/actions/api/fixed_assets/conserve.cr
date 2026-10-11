class Api::FixedAssets::Conserve < ApiAction
  post "/api/companies/:company_id/fixed_assets/:fixed_asset_id/conserve" do
    asset = FixedAssetQuery.new.company_id(company_id).find(fixed_asset_id)
    body = request.body.try(&.gets_to_end) || "{}"
    json_data = JSON.parse(body)
    from_str = json_data["conserved_from"]?.try(&.as_s)
    to_str = json_data["conserved_to"]?.try(&.as_s)
    from = from_str.try { |s| Time.parse(s, "%Y-%m-%d", Time::Location::UTC) }
    to = to_str.try { |s| Time.parse(s, "%Y-%m-%d", Time::Location::UTC) }

    if from.nil?
      response.status_code = 422
      json({success: false, message: "Посочете начална дата на консервацията"})
    else
      new_status = if to && to < Time.utc
        "active"
      else
        "conserved"
      end
      SaveFixedAsset.update!(asset,
        conserved_from: from,
        conserved_to: to,
        status: new_status
      )
      json({success: true, data: FixedAssetQuery.new.find(asset.id).api_json})
    end
  end
end
