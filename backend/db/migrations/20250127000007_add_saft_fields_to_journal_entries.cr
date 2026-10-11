class AddSaftFieldsToJournalEntries::V20250127000007 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(JournalEntry) do
      # SAF-T Transaction fields
      add transaction_id : String?          # Unique transaction ID (required for SAF-T)
      add period : Int32?                   # Accounting period (1-12)
      add period_year : Int32?              # Year of accounting period
      add journal_id : String?              # Journal identifier (e.g., PAYMENTS, SALES)
      add journal_type : String?            # Journal type (e.g., AP, AR)
      add transaction_type : String?        # Transaction type (normal, periodic, etc.)
      add batch_id : String?                # Batch ID
      add system_entry_date : Time?         # System entry date
      add gl_posting_date : Time?           # GL posting date
      add source_id : String?               # Source (user/module that created)

      # SAF-T CustomerID/SupplierID (formatted)
      add saft_customer_id : String?
      add saft_supplier_id : String?
    end

    create_index :journal_entries, [:company_id, :transaction_id]
  end

  def rollback
    alter table_for(JournalEntry) do
      remove :transaction_id
      remove :period
      remove :period_year
      remove :journal_id
      remove :journal_type
      remove :transaction_type
      remove :batch_id
      remove :system_entry_date
      remove :gl_posting_date
      remove :source_id
      remove :saft_customer_id
      remove :saft_supplier_id
    end
  end
end
