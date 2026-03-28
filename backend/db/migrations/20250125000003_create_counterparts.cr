class CreateCounterparts::V20250125000003 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(Counterpart) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add name : String
      add vat_number : String?
      add address : String?
      add contact_person : String?
      add email : String?
      add phone : String?
      add counterpart_type : String
    end

    create_index :counterparts, [:company_id, :vat_number]
  end

  def rollback
    drop table_for(Counterpart)
  end
end
