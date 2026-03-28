class CreateIsoCurrencies::V20250126000017 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(IsoCurrency) do
      primary_key id : Int64
      add_timestamps
      add code : String
      add name : String
      add name_bg : String?
      add numeric : String?
      add symbol : String?
      add decimal_places : Int32, default: 2
    end

    create_index :iso_currencies, [:code], unique: true
  end

  def rollback
    drop table_for(IsoCurrency)
  end
end
