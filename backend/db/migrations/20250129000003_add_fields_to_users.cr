class AddFieldsToUsers::V20250129000003 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(User) do
      add first_name : String?
      add last_name : String?
      add is_active : Bool, default: true
    end
  end

  def rollback
    alter table_for(User) do
      remove :first_name
      remove :last_name
      remove :is_active
    end
  end
end
