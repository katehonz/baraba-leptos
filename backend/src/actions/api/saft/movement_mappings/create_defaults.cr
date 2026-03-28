# POST /api/companies/:company_id/saft/movement_mappings/create_defaults
class Api::Saft::MovementMappings::CreateDefaults < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/companies/:company_id/saft/movement_mappings/create_defaults" do
    # Check if company already has mappings
    existing = SaftMovementAccountMappingQuery.new.company_id(company_id).select_count

    if existing > 0
      return json({success: false, error: "Company already has #{existing} mappings. Delete them first."})
    end

    SaftMovementMapper.create_default_mappings(company_id.to_i64)

    count = SaftMovementAccountMappingQuery.new.company_id(company_id).select_count

    json({success: true, message: "Created #{count} default mappings"})
  rescue ex
    json({success: false, error: ex.message})
  end
end
