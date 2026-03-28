# POST /api/companies/:company_id/saft/asset_mappings/create_defaults
class Api::Saft::AssetMappings::CreateDefaults < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/companies/:company_id/saft/asset_mappings/create_defaults" do
    # Check if company already has mappings
    existing = SaftAssetAccountMappingQuery.new.company_id(company_id).select_count

    if existing > 0
      return json({success: false, error: "Фирмата вече има #{existing} настройки. Изтрийте ги първо."})
    end

    SaftAssetMapper.create_default_mappings(company_id.to_i64)

    count = SaftAssetAccountMappingQuery.new.company_id(company_id).select_count

    json({success: true, message: "Създадени #{count} стандартни настройки"})
  rescue ex
    json({success: false, error: ex.message})
  end
end
