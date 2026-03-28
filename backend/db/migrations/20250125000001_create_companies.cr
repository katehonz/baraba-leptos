class CreateCompanies::V20250125000001 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(Company) do
      primary_key id : Int64
      add_timestamps
      add name : String
      add vat_number : String?
      add address : String?
    end

    create_index :companies, :vat_number, unique: true
  end

  def rollback
    drop table_for(Company)
  end
end
