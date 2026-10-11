class CreateSaftTaxTypes::V20250126000015 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(SaftTaxType) do
      primary_key id : Int64
      add_timestamps
      add code : String
      add name : String
      add description : String?
    end

    create_index :saft_tax_types, [:code], unique: true
  end

  def rollback
    drop table_for(SaftTaxType)
  end
end
