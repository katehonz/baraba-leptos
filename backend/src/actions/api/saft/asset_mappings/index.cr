# GET /api/companies/:company_id/saft/asset_mappings
class Api::Saft::AssetMappings::Index < ApiAction
  include Api::Auth::SkipRequireAuthToken

  get "/api/companies/:company_id/saft/asset_mappings" do
    mappings = SaftAssetAccountMappingQuery.new
      .company_id(company_id)
      .asset_transaction_type.asc_order
      .debit_account.asc_order

    # Get transaction type names
    transaction_types = SaftAssetMovementTypeQuery.new.to_a.to_h { |t| {t.code, t.name_bg} }

    json({
      success: true,
      data:    mappings.map { |m|
        {
          id:                      m.id,
          asset_transaction_type:  m.asset_transaction_type,
          transaction_type_name:   transaction_types[m.asset_transaction_type]? || m.asset_transaction_type,
          debit_account:           m.debit_account,
          credit_account:          m.credit_account,
          debit_analytical:        m.debit_analytical,
          credit_analytical:       m.credit_analytical,
          description:             m.description,
          is_active:               m.is_active,
        }
      },
      transaction_types: transaction_types,
    })
  end
end
