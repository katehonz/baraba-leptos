class CreateUserCompanyRoles::V20250126000025 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(UserCompanyRole) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to user : User, on_delete: :cascade
      add_belongs_to company : Company, on_delete: :cascade
      add_belongs_to role : Role, on_delete: :cascade
    end

    create_index :user_company_roles, [:user_id, :company_id], unique: true
  end

  def rollback
    drop table_for(UserCompanyRole)
  end
end
