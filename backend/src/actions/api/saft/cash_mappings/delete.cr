# DELETE /api/companies/:company_id/saft/cash_mappings/:id
class Api::Saft::CashMappings::Delete < ApiAction
  include Api::Auth::SkipRequireAuthToken

  delete "/api/companies/:company_id/saft/cash_mappings/:id" do
    mapping = SaftCashAccountMappingQuery.new
      .company_id(company_id)
      .id(id)
      .first?

    unless mapping
      return json({success: false, error: "Mapping not found"})
    end

    mapping.delete

    json({success: true})
  rescue ex
    json({success: false, error: ex.message})
  end
end
