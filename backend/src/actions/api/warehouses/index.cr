class Api::Warehouses::Index < ApiAction
  get "/api/companies/:company_id/warehouses" do
    warehouses = WarehouseQuery.new.company_id(company_id)

    if params.get?("active") == "true"
      warehouses = warehouses.is_active(true)
    end

    json({
      success: true,
      data:    warehouses.map { |w| {
        id:          w.id,
        name:        w.name,
        address:     w.address,
        description: w.description,
        is_active:   w.is_active,
      } },
    })
  end
end
