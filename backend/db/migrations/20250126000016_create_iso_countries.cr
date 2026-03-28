class CreateIsoCountries::V20250126000016 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(IsoCountry) do
      primary_key id : Int64
      add_timestamps
      add alpha2 : String
      add alpha3 : String
      add name : String
      add name_bg : String?
      add numeric : String?
      add is_eu_member : Bool, default: false
    end

    create_index :iso_countries, [:alpha2], unique: true
    create_index :iso_countries, [:alpha3], unique: true
  end

  def rollback
    drop table_for(IsoCountry)
  end
end
