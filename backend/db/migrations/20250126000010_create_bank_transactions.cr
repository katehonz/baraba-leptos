class CreateBankTransactions::V20250126000010 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(BankTransaction) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to bank_account : BankAccount, on_delete: :cascade
      add_belongs_to journal_entry : JournalEntry?, on_delete: :nullify
      add date : Time
      add amount : Float64
      add currency : String?
      add amount_base : Float64?
      add exchange_rate : Float64?
      add description : String?
      add contra_account : String?
      add contra_name : String?
      add reference : String?
      add salt_edge_transaction_id : String?
      add salt_edge_category : String?
      add wise_transaction_id : String?
    end

    create_index :bank_transactions, [:bank_account_id, :date]
    create_index :bank_transactions, [:salt_edge_transaction_id], unique: true
    create_index :bank_transactions, [:wise_transaction_id], unique: true
  end

  def rollback
    drop table_for(BankTransaction)
  end
end
