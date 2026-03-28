class CreateWarehouses::V20250126000008 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(Warehouse) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add name : String
      add address : String?
      add description : String?
      add is_active : Bool, default: true
    end

    create_index :warehouses, [:company_id, :name], unique: true
  end

  def rollback
    drop table_for(Warehouse)
  end
end
