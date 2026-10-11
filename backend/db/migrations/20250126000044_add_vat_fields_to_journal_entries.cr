class AddVatFieldsToJournalEntries::V20250126000044 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(JournalEntry) do
      add document_number : String?
      add document_date : Time?
      add vat_period : String?              # "2025-01" format
      add vat_purchase_operation : String?   # VAT purchase operation type
      add vat_sales_operation : String?     # VAT sales operation type
      add total_amount : Float64?
      add total_vat_amount : Float64?
      add payment_method_code : String?     # SAFT payment method code
    end

    create_index :journal_entries, [:company_id, :vat_period], unique: false
    create_index :journal_entries, [:company_id, :document_number], unique: false
  end

  def rollback
    drop_index :journal_entries, [:company_id, :document_number]
    drop_index :journal_entries, [:company_id, :vat_period]

    alter table_for(JournalEntry) do
      remove :document_number
      remove :document_date
      remove :vat_period
      remove :vat_purchase_operation
      remove :vat_sales_operation
      remove :total_amount
      remove :total_vat_amount
      remove :payment_method_code
    end
  end
end
