class CreatePayments::V20250127000008 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(Payment) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add_belongs_to counterpart : Counterpart?, on_delete: :nullify
      add_belongs_to journal_entry : JournalEntry?, on_delete: :nullify
      add_belongs_to bank_account : BankAccount?, on_delete: :nullify

      # SAF-T Payment fields
      add payment_ref_no : String           # Unique reference number
      add period : Int32?                   # Accounting period (1-12)
      add period_year : Int32?              # Year of accounting period
      add transaction_id : String           # Transaction ID (links to GL)
      add transaction_date : Time           # Payment date
      add payment_method : String           # 01=Cash, 02=Offset, 03=Cashless
      add description : String
      add batch_id : String?
      add system_id : String?
      add source_id : String?

      # SAF-T CustomerID/SupplierID (formatted)
      add saft_customer_id : String?
      add saft_supplier_id : String?

      # Totals
      add net_total : Float64?
      add gross_total : Float64
      add currency_code : String, default: "BGN"
      add exchange_rate : Float64, default: 1.0
    end

    create_index :payments, [:company_id, :payment_ref_no], unique: true
    create_index :payments, [:company_id, :transaction_id]
  end

  def rollback
    drop table_for(Payment)
  end
end
