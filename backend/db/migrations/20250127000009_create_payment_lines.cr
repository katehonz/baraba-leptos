class CreatePaymentLines::V20250127000009 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(PaymentLine) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to payment : Payment, on_delete: :cascade
      add_belongs_to account : Account?, on_delete: :nullify
      add_belongs_to invoice : Invoice?, on_delete: :nullify

      # SAF-T PaymentLine fields
      add line_number : Int32?
      add source_document_id : String?      # Invoice number or other document
      add description : String?
      add debit_credit_indicator : String   # D or C
      add tax_point_date : Time?

      # Amount
      add amount : Float64
      add currency_code : String, default: "BGN"
      add currency_amount : Float64?
      add exchange_rate : Float64, default: 1.0

      # SAF-T CustomerID/SupplierID (formatted)
      add saft_customer_id : String?
      add saft_supplier_id : String?

      # Tax information
      add tax_type : String?
      add tax_code : String?
      add tax_percentage : Float64?
      add tax_base : Float64?
      add tax_amount : Float64?
    end
    # Note: indexes are automatically created by add_belongs_to
  end

  def rollback
    drop table_for(PaymentLine)
  end
end
