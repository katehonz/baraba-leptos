class CreateFixedAssetTransactions::V20250126000023 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(FixedAssetTransaction) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to fixed_asset : FixedAsset, on_delete: :cascade
      add_belongs_to company : Company, on_delete: :cascade
      add transaction_date : Time
      add amount : Float64
      add movement_type : String?
      add description : String?
      add document_number : String?
      add document_date : Time?
    end

    create_index :fixed_asset_transactions, [:fixed_asset_id, :transaction_date]
  end

  def rollback
    drop table_for(FixedAssetTransaction)
  end
end
