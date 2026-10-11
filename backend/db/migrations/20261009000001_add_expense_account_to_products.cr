class AddExpenseAccountToProducts::V20261009000001 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(Product) do
      add_belongs_to expense_account : Account?, on_delete: :nullify
    end
  end

  def rollback
    alter table_for(Product) do
      remove_belongs_to :expense_account
    end
  end
end
