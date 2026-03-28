class CreateAccounts::V20250125000002 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(Account) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add code : String
      add name : String
      add account_type : String
      add is_active : Bool, default: true
    end

    create_index :accounts, [:company_id, :code], unique: true
  end

  def rollback
    drop table_for(Account)
  end
end
