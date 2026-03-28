class CreateScannedInvoices::V20250126000022 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(ScannedInvoice) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add_belongs_to journal_entry : JournalEntry?, on_delete: :nullify
      add_belongs_to counterpart : Counterpart?, on_delete: :nullify

      add direction : String
      add status : String, default: "pending"

      # Vendor info
      add vendor_name : String?
      add vendor_vat_number : String?
      add vendor_address : String?

      # Customer info
      add customer_name : String?
      add customer_vat_number : String?
      add customer_address : String?

      # Invoice details
      add invoice_number : String?
      add invoice_date : Time?
      add due_date : Time?

      # Amounts
      add subtotal : Float64?
      add total_tax : Float64?
      add invoice_total : Float64?

      # VIES validation
      add vies_status : String, default: "pending"
      add vies_validation_message : String?
      add vies_company_name : String?
      add vies_company_address : String?
      add vies_validated_at : Time?

      # Selected accounts
      add counterparty_account_id : Int64?
      add vat_account_id : Int64?
      add expense_revenue_account_id : Int64?

      # Review flags
      add requires_manual_review : Bool, default: false
      add manual_review_reason : String?
      add notes : String?

      # Processing metadata
      add confidence : Float64?
      add original_file_name : String?
      add s3_key : String?
      add s3_key_json : String?
      add azure_raw_json : String?
      add vat_period : String?
      add has_inventory : Bool, default: false
    end

    create_index :scanned_invoices, [:company_id, :status]
    create_index :scanned_invoices, [:company_id, :vat_period]
  end

  def rollback
    drop table_for(ScannedInvoice)
  end
end
