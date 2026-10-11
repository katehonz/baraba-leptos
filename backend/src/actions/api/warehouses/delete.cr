class Api::Warehouses::Delete < ApiAction
  delete "/api/companies/:company_id/warehouses/:warehouse_id" do
    warehouse = WarehouseQuery.new.company_id(company_id).find(warehouse_id)
    DeleteWarehouse.delete!(warehouse)
    json({success: true, message: "Warehouse deleted successfully"})
  end
end
