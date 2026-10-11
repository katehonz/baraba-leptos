class AddBufferAccountToBankAccounts::V20260214000002 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(BankAccount) do
      add_belongs_to buffer_account : Account?, on_delete: :nullify
    end
  end

  def rollback
    alter table_for(BankAccount) do
      remove_belongs_to :buffer_account
    end
  end
end
