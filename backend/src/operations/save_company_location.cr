class SaveCompanyLocation < CompanyLocation::SaveOperation
  permit_columns name, location_type, street_name, building_number,
    city, post_code, region, country, phone, email,
    is_main, is_active, company_id

  before_save do
    validate_required name
    validate_required location_type
    validate_inclusion_of location_type, in: ["OFFICE", "STORE", "WAREHOUSE", "BRANCH"]
  end
end
