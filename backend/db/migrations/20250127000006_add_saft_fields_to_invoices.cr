class AddSaftFieldsToInvoices::V20250127000006 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(Invoice) do
      # SAF-T Invoice fields
      add saft_invoice_type : String?       # Code from Nom_Invoice_Types (1, 2, 3, 7, 9, etc.)
      add period : Int32?                   # Accounting period (1-12)
      add period_year : Int32?              # Year of accounting period
      add transaction_id : String?          # Unique transaction ID (required for SAF-T)
      add branch_store_number : String?     # Branch/store number
      add self_billing_indicator : Bool, default: false
      add gl_posting_date : Time?           # GL posting date
      add batch_id : String?                # Batch ID
      add system_id : String?               # System ID
      add source_id : String?               # Source (user/module that created)

      # SAF-T CustomerID/SupplierID (formatted)
      add saft_customer_id : String?
      add saft_supplier_id : String?
    end

    create_index :invoices, [:company_id, :transaction_id]
  end

  def rollback
    alter table_for(Invoice) do
      remove :saft_invoice_type
      remove :period
      remove :period_year
      remove :transaction_id
      remove :branch_store_number
      remove :self_billing_indicator
      remove :gl_posting_date
      remove :batch_id
      remove :system_id
      remove :source_id
      remove :saft_customer_id
      remove :saft_supplier_id
    end
  end
end
