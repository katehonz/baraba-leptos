class CreateStockTransactions::V20250126000009 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(StockTransaction) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to product : Product, on_delete: :cascade
      add_belongs_to company : Company, on_delete: :cascade
      add_belongs_to warehouse : Warehouse, on_delete: :cascade
      add_belongs_to journal_entry : JournalEntry?, on_delete: :nullify
      add transaction_date : Time
      add quantity : Float64
      add unit_price : Float64
      add total_amount : Float64
      add is_incoming : Bool
      add movement_type : String?
      add document_number : String?
      add document_date : Time?
    end

    create_index :stock_transactions, [:product_id, :transaction_date]
    create_index :stock_transactions, [:warehouse_id, :transaction_date]
  end

  def rollback
    drop table_for(StockTransaction)
  end
end
