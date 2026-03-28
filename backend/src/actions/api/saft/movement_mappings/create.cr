# POST /api/companies/:company_id/saft/movement_mappings
class Api::Saft::MovementMappings::Create < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/companies/:company_id/saft/movement_mappings" do
    body = parse_json_body

    mapping = SaveSaftMovementAccountMapping.create!(
      company_id: company_id.to_i64,
      movement_type_code: body["movement_type_code"].as_s,
      debit_account: body["debit_account"].as_s,
      credit_account: body["credit_account"].as_s,
      debit_analytical: body["debit_analytical"]?.try(&.as_s?),
      credit_analytical: body["credit_analytical"]?.try(&.as_s?),
      description: body["description"]?.try(&.as_s?),
      is_active: body["is_active"]?.try(&.as_bool?) || true
    )

    json({success: true, data: {id: mapping.id}})
  rescue ex
    json({success: false, error: ex.message})
  end

  private def parse_json_body : JSON::Any
    body_str = request.body.try(&.gets_to_end) || "{}"
    JSON.parse(body_str)
  end
end
