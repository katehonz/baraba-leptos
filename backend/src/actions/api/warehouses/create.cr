class Api::Warehouses::Create < ApiAction
  post "/api/companies/:company_id/warehouses" do
    SaveWarehouse.create(params, company_id: company_id.to_i64) do |operation, warehouse|
      if warehouse
        json({
          success: true,
          data:    {
            id:          warehouse.id,
            name:        warehouse.name,
            address:     warehouse.address,
            description: warehouse.description,
            is_active:   warehouse.is_active,
          },
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
