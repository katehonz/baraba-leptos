class Api::Warehouses::Create < ApiAction
  post "/api/companies/:company_id/warehouses" do
    response.status_code = 410
    json({
      success: false,
      error:   "Складовете се управляват през търговски обекти (/locations). Поставете holds_stock=true.",
    })
  end
end
