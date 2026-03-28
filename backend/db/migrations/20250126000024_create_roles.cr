class CreateRoles::V20250126000024 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(Role) do
      primary_key id : Int64
      add_timestamps
      add name : String
      add description : String?
      add permissions : String?
    end

    create_index :roles, [:name], unique: true
  end

  def rollback
    drop table_for(Role)
  end
end
