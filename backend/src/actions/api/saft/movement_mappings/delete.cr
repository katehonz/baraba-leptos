# DELETE /api/companies/:company_id/saft/movement_mappings/:mapping_id
class Api::Saft::MovementMappings::Delete < ApiAction
  include Api::Auth::SkipRequireAuthToken

  delete "/api/companies/:company_id/saft/movement_mappings/:mapping_id" do
    mapping = SaftMovementAccountMappingQuery.new
      .company_id(company_id)
      .id(mapping_id)
      .first?

    unless mapping
      return json({success: false, error: "Mapping not found"})
    end

    mapping.delete

    json({success: true, message: "Deleted"})
  rescue ex
    json({success: false, error: ex.message})
  end
end
