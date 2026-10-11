class Api::FixedAssets::Create < ApiAction
  post "/api/companies/:company_id/fixed_assets" do
    SaveFixedAsset.create(params, company_id: company_id.to_i64) do |operation, asset|
      if asset
        json({success: true, data: asset.api_json})
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
