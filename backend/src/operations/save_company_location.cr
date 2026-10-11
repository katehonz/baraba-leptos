class SaveCompanyLocation < CompanyLocation::SaveOperation
  permit_columns name, location_type, street_name, building_number,
    city, post_code, region, country, phone, email,
    is_main, is_active, company_id, holds_stock, branch_code, saft_location_code

  before_save do
    validate_required name
    validate_required location_type
    validate_inclusion_of location_type, in: ["OFFICE", "STORE", "WAREHOUSE", "BRANCH"]
    if holds_stock.value.nil?
      holds_stock.value = ["WAREHOUSE", "STORE"].includes?(location_type.value)
    end
  end

  after_save ensure_warehouse

  private def ensure_warehouse(location : CompanyLocation)
    return unless location.holds_stock
    existing = WarehouseQuery.new.company_location_id(location.id).first?
    if existing
      SaveWarehouse.update!(existing, name: location.name, is_active: location.is_active)
    else
      SaveWarehouse.create!(
        company_id: location.company_id,
        company_location_id: location.id,
        name: location.name,
        is_active: location.is_active
      )
    end
  end
end
