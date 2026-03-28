# GET /api/companies/:company_id/saft/cash_mappings
class Api::Saft::CashMappings::Index < ApiAction
  include Api::Auth::SkipRequireAuthToken

  get "/api/companies/:company_id/saft/cash_mappings" do
    mappings = SaftCashAccountMappingQuery.new
      .company_id(company_id)
      .cash_movement_type.asc_order
      .debit_account.asc_order

    # Get cash movement type names
    movement_types = SaftCashMovementTypeQuery.new.to_a.to_h { |t| {t.code, t.name_bg} }

    json({
      success: true,
      data:    mappings.map { |m|
        {
          id:                   m.id,
          cash_movement_type:   m.cash_movement_type,
          movement_type_name:   movement_types[m.cash_movement_type]? || m.cash_movement_type,
          debit_account:        m.debit_account,
          credit_account:       m.credit_account,
          debit_analytical:     m.debit_analytical,
          credit_analytical:    m.credit_analytical,
          description:          m.description,
          is_active:            m.is_active,
        }
      },
      movement_types: movement_types,
    })
  end
end
