class AddSubledgerToOpeningBalances::V20250129000001 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(OpeningBalance) do
      add counterpart_id : Int64?, index: true
      add product_id : Int64?, index: true
      add warehouse_id : Int64?, index: true
      add quantity : Float64?
      add description : String?
      add balance_type : String, default: "account"  # account, counterpart, product
    end

    # Add foreign keys
    execute "ALTER TABLE opening_balances ADD CONSTRAINT fk_opening_balances_counterpart FOREIGN KEY (counterpart_id) REFERENCES counterparts(id) ON DELETE SET NULL"
    execute "ALTER TABLE opening_balances ADD CONSTRAINT fk_opening_balances_product FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE SET NULL"
    execute "ALTER TABLE opening_balances ADD CONSTRAINT fk_opening_balances_warehouse FOREIGN KEY (warehouse_id) REFERENCES warehouses(id) ON DELETE SET NULL"

    # Update unique index to include subledger fields
    execute "DROP INDEX IF EXISTS opening_balances_company_id_account_id_date_index"
    execute "CREATE UNIQUE INDEX opening_balances_unique_idx ON opening_balances (company_id, account_id, date, COALESCE(counterpart_id, 0), COALESCE(product_id, 0), COALESCE(warehouse_id, 0))"
  end

  def rollback
    execute "DROP INDEX IF EXISTS opening_balances_unique_idx"
    execute "ALTER TABLE opening_balances DROP CONSTRAINT IF EXISTS fk_opening_balances_counterpart"
    execute "ALTER TABLE opening_balances DROP CONSTRAINT IF EXISTS fk_opening_balances_product"
    execute "ALTER TABLE opening_balances DROP CONSTRAINT IF EXISTS fk_opening_balances_warehouse"

    alter table_for(OpeningBalance) do
      remove :counterpart_id
      remove :product_id
      remove :warehouse_id
      remove :quantity
      remove :description
      remove :balance_type
    end

    create_index :opening_balances, [:company_id, :account_id, :date], unique: true
  end
end
