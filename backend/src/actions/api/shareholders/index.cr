class Api::Shareholders::Index < ApiAction
  get "/api/companies/:company_id/shareholders" do
    shareholders = ShareholderQuery.new.company_id(company_id)

    json({
      success: true,
      data:    shareholders.map { |s| {
        id:               s.id,
        shareholder_type: s.shareholder_type,
        name:             s.name,
        identifier:       s.identifier,
        address:          s.address,
        percentage_owned: s.percentage_owned,
      } },
    })
  end
end
