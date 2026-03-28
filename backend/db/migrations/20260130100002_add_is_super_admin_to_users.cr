class AddIsSuperAdminToUsers::V20260130100002 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(User) do
      add is_super_admin : Bool, default: false
    end
  end

  def rollback
    alter table_for(User) do
      remove :is_super_admin
    end
  end
end
