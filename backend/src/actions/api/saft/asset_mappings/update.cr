# PUT /api/companies/:company_id/saft/asset_mappings/:mapping_id
class Api::Saft::AssetMappings::Update < ApiAction
  include Api::Auth::SkipRequireAuthToken

  put "/api/companies/:company_id/saft/asset_mappings/:mapping_id" do
    mapping = SaftAssetAccountMappingQuery.new
      .company_id(company_id)
      .id(mapping_id)
      .first?

    unless mapping
      return json({success: false, error: "Mapping not found"})
    end

    body = parse_json_body

    SaveSaftAssetAccountMapping.update!(mapping,
      asset_transaction_type: body["asset_transaction_type"]?.try(&.as_s?) || mapping.asset_transaction_type,
      debit_account: body["debit_account"]?.try(&.as_s?) || mapping.debit_account,
      credit_account: body["credit_account"]?.try(&.as_s?) || mapping.credit_account,
      debit_analytical: body["debit_analytical"]?.try(&.as_s?),
      credit_analytical: body["credit_analytical"]?.try(&.as_s?),
      description: body["description"]?.try(&.as_s?),
      is_active: body["is_active"]?.try(&.as_bool?) || mapping.is_active
    )

    json({success: true, message: "Updated"})
  rescue ex
    json({success: false, error: ex.message})
  end

  private def parse_json_body : JSON::Any
    body_str = request.body.try(&.gets_to_end) || "{}"
    JSON.parse(body_str)
  end
end
