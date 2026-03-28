class AddSaftFieldsToStockTransactions::V20250127000010 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(StockTransaction) do
      # SAF-T MovementOfGoods fields
      add movement_reference : String?      # Unique movement ID
      add movement_posting_date : Time?
      add movement_posting_time : Time?
      add tax_point_date : Time?
      # Note: movement_type already exists from original migration
      add movement_subtype : String?        # Subtype description
      add source_id : String?               # User/system that created
      add system_id : String?

      # Document reference
      add document_type : String?
      # Note: document_number already exists from original migration
      add document_line : String?

      # SAF-T Line fields
      add line_number : Int32?
      add_belongs_to account : Account?, on_delete: :nullify
      add transaction_id : String?          # Links to GL

      # SAF-T CustomerID/SupplierID
      add saft_customer_id : String?
      add saft_supplier_id : String?

      # Shipping points (JSON for ShippingPointStructure)
      add ship_to : String?
      add ship_from : String?

      # Stock fields
      add stock_account_no : String?        # Batch/serial number
      add unit_of_measure : String?         # SAF-T code
      add uom_conversion_factor : Float64, default: 1.0
      add book_value : Float64?
      add movement_comments : String?

      # Tax information
      add tax_type : String?
      add tax_code : String?
      add tax_percentage : Float64?
      add tax_base : Float64?
      add tax_amount : Float64?
    end

    create_index :stock_transactions, [:company_id, :movement_reference]
  end

  def rollback
    alter table_for(StockTransaction) do
      remove :movement_reference
      remove :movement_posting_date
      remove :movement_posting_time
      remove :tax_point_date
      # Note: movement_type was in original migration
      remove :movement_subtype
      remove :source_id
      remove :system_id
      remove :document_type
      # Note: document_number was in original migration
      remove :document_line
      remove :line_number
      remove :account_id
      remove :transaction_id
      remove :saft_customer_id
      remove :saft_supplier_id
      remove :ship_to
      remove :ship_from
      remove :stock_account_no
      remove :unit_of_measure
      remove :uom_conversion_factor
      remove :book_value
      remove :movement_comments
      remove :tax_type
      remove :tax_code
      remove :tax_percentage
      remove :tax_base
      remove :tax_amount
    end
  end
end
