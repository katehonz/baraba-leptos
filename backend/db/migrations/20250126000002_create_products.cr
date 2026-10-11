class CreateProducts::V20250126000002 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(Product) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add_belongs_to inventory_account : Account?, on_delete: :nullify
      add_belongs_to revenue_account : Account?, on_delete: :nullify
      add code : String
      add name : String
      add description : String?
      add product_type : String?
      add price : Float64, default: 0.0
      add measure_unit : String?
      add commodity_code : String?
      add tax_type : String?
      add tax_code : String?
      add is_active : Bool, default: true
    end

    create_index :products, [:company_id, :code], unique: true
  end

  def rollback
    drop table_for(Product)
  end
end
