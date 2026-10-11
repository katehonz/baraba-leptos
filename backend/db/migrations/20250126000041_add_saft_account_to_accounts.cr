class AddSaftAccountToAccounts::V20250126000041 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(Account) do
      add_belongs_to saft_account : SaftAccount?, on_delete: :nullify
    end
  end

  def rollback
    alter table_for(Account) do
      remove_belongs_to :saft_account
    end
  end
end
