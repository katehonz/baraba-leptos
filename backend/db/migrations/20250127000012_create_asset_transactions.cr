class CreateAssetTransactions::V20250127000012 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(AssetTransaction) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add_belongs_to fixed_asset : FixedAsset, on_delete: :cascade
      add_belongs_to counterpart : Counterpart?, on_delete: :nullify
      add_belongs_to journal_entry : JournalEntry?, on_delete: :nullify

      # SAF-T AssetTransaction fields
      add asset_transaction_id : String     # Unique ID from system
      add asset_transaction_type : String   # Code from AssetMovementTypes
      add description : String?
      add transaction_date : Time
      add transaction_id : String?          # Links to GeneralLedgerEntries

      # SAF-T CustomerID/SupplierID
      add saft_customer_id : String?
      add saft_supplier_id : String?

      # Valuation fields
      add valuation_type : String?          # Valuation method
      add acquisition_cost_on_transaction : Float64?
      add book_value_on_transaction : Float64
      add transaction_amount : Float64
    end

    create_index :asset_transactions, [:company_id, :asset_transaction_id], unique: true
    # Note: fixed_asset_id index is auto-created by add_belongs_to
  end

  def rollback
    drop table_for(AssetTransaction)
  end
end
