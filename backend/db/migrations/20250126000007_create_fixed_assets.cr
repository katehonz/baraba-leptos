class CreateFixedAssets::V20250126000007 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(FixedAsset) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add_belongs_to category : FixedAssetCategory?, on_delete: :nullify
      add inventory_number : String
      add name : String
      add description : String?
      add acquisition_date : Time
      add put_into_service_date : Time?
      add acquisition_cost : Float64
      add residual_value : Float64?
      add accounting_book_value : Float64?
      add depreciation_method : String
      add accounting_depreciation_rate : Float64
      add tax_depreciation_rate : Float64
      add status : String, default: "active"
      add document_number : String?
      add document_date : Time?
    end

    create_index :fixed_assets, [:company_id, :inventory_number], unique: true
  end

  def rollback
    drop table_for(FixedAsset)
  end
end
