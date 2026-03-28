class CreateInvoices::V20250125000004 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(Invoice) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add_belongs_to counterpart : Counterpart, on_delete: :restrict
      add invoice_number : String
      add invoice_date : Time
      add due_date : Time?
      add subtotal : Float64
      add vat_amount : Float64
      add total_amount : Float64
      add currency : String, default: "BGN"
      add status : String, default: "draft"
      add notes : String?
    end

    create_index :invoices, [:company_id, :invoice_number], unique: true
  end

  def rollback
    drop table_for(Invoice)
  end
end
