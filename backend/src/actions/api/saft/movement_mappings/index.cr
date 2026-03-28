# GET /api/companies/:company_id/saft/movement_mappings
class Api::Saft::MovementMappings::Index < ApiAction
  include Api::Auth::SkipRequireAuthToken

  get "/api/companies/:company_id/saft/movement_mappings" do
    mappings = SaftMovementAccountMappingQuery.new
      .company_id(company_id)
      .movement_type_code.asc_order
      .debit_account.asc_order

    # Get movement type names
    movement_types = SaftStockMovementTypeQuery.new.to_a.to_h { |t| {t.code, t.name_bg} }

    json({
      success: true,
      data:    mappings.map { |m|
        {
          id:                 m.id,
          movement_type_code: m.movement_type_code,
          movement_type_name: movement_types[m.movement_type_code]? || m.movement_type_code,
          debit_account:      m.debit_account,
          credit_account:     m.credit_account,
          debit_analytical:   m.debit_analytical,
          credit_analytical:  m.credit_analytical,
          description:        m.description,
          is_active:          m.is_active,
        }
      },
      movement_types: movement_types,
    })
  end
end
