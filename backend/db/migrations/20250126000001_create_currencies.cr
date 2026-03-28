class CreateCurrencies::V20250126000001 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(Currency) do
      primary_key id : Int64
      add_timestamps
      add code : String
      add name : String
      add name_bg : String?
      add symbol : String?
      add decimal_places : Int32, default: 2
      add is_active : Bool, default: true
      add is_base_currency : Bool, default: false
    end

    create_index :currencies, [:code], unique: true
  end

  def rollback
    drop table_for(Currency)
  end
end
