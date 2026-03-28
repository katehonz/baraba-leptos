class CreateCompanyLocations::V20260201000003 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(CompanyLocation) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade

      add name : String                       # Име на обекта (напр. "Магазин Витоша")
      add location_type : String              # OFFICE, STORE, WAREHOUSE, BRANCH
      add street_name : String?               # Улица
      add building_number : String?           # Номер
      add city : String?                      # Град
      add post_code : String?                 # Пощенски код
      add region : String?                    # ISO 3166-2:BG (напр. BG-23)
      add country : String, default: "BG"     # Държава
      add phone : String?                     # Телефон
      add email : String?                     # Email
      add is_main : Bool, default: false      # Главен адрес
      add is_active : Bool, default: true
    end
  end

  def rollback
    drop table_for(CompanyLocation)
  end
end
