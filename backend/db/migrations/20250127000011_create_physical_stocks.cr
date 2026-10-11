class CreatePhysicalStocks::V20250127000011 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(PhysicalStock) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add_belongs_to warehouse : Warehouse, on_delete: :cascade
      add_belongs_to product : Product, on_delete: :cascade

      # SAF-T PhysicalStock fields
      add location_id : String?             # Location within warehouse
      add stock_account_no : String?        # Batch/serial number
      add product_type : String             # 10, 20, 30, 40, 50 (from Nom_Inventory_Types)
      add product_status : String?          # active, damaged, obsolete, etc.
      add stock_commodity_code : String?    # NC8_TARIC code

      # Owner (SAF-T OwnerID format)
      add owner_id : String                 # SAF-T formatted owner ID

      # Unit of measure
      add uom_physical_stock : String       # SAF-T code
      add uom_conversion_factor : Float64, default: 1.0

      # Prices and quantities
      add unit_price_begin : Float64, default: 0.0
      add unit_price_end : Float64, default: 0.0
      add opening_stock_quantity : Float64, default: 0.0
      add opening_stock_value : Float64, default: 0.0
      add closing_stock_quantity : Float64, default: 0.0
      add closing_stock_value : Float64, default: 0.0

      # Reporting period
      add period_start : Time?
      add period_end : Time?

      # Stock characteristics (JSON)
      add characteristics : String?         # JSON array of {code, value} pairs
    end

    create_index :physical_stocks, [:company_id, :warehouse_id, :product_id]
    create_index :physical_stocks, [:company_id, :owner_id]
  end

  def rollback
    drop table_for(PhysicalStock)
  end
end
