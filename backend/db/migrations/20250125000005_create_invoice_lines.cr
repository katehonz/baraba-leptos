class CreateInvoiceLines::V20250125000005 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(InvoiceLine) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to invoice : Invoice, on_delete: :cascade
      add_belongs_to account : Account?, on_delete: :nullify
      add description : String
      add quantity : Float64
      add unit_price : Float64
      add vat_rate : Float64, default: 20.0
      add line_total : Float64
    end
  end

  def rollback
    drop table_for(InvoiceLine)
  end
end
