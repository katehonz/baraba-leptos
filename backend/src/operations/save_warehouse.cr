class SaveWarehouse < Warehouse::SaveOperation
  permit_columns name, address, description, is_active, company_id

  before_save do
    validate_required name
  end
end
