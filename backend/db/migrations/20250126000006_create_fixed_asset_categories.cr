class CreateFixedAssetCategories::V20250126000006 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(FixedAssetCategory) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add_belongs_to asset_account : Account?, on_delete: :nullify
      add_belongs_to depreciation_account : Account?, on_delete: :nullify
      add_belongs_to expense_account : Account?, on_delete: :nullify
      add name : String
      add description : String?
      add cita_category : String?
      add min_depreciation_rate : Float64
      add max_depreciation_rate : Float64
      add default_method : String, default: "straight_line"
    end

    create_index :fixed_asset_categories, [:company_id, :name], unique: true
  end

  def rollback
    drop table_for(FixedAssetCategory)
  end
end
