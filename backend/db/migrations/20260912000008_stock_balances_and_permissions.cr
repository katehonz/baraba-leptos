class StockBalancesAndPermissions::V20260912000008 < Avram::Migrator::Migration::V1
  def migrate
    execute <<-SQL
      CREATE TABLE stock_balances (
        id BIGSERIAL PRIMARY KEY,
        company_id BIGINT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
        product_id BIGINT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
        warehouse_id BIGINT NOT NULL REFERENCES warehouses(id) ON DELETE CASCADE,
        qty NUMERIC NOT NULL DEFAULT 0,
        avg_cost NUMERIC NOT NULL DEFAULT 0,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        UNIQUE (company_id, product_id, warehouse_id)
      );
    SQL

    execute <<-SQL
      ALTER TABLE companies ALTER COLUMN inventory_valuation_method SET DEFAULT 'WEIGHTED_AVG';
    SQL

    execute <<-SQL
      UPDATE roles
         SET permissions = (
           SELECT COALESCE(jsonb_agg(DISTINCT val), '[]'::jsonb)
             FROM (
               SELECT jsonb_array_elements_text(permissions::jsonb) AS val
               UNION ALL SELECT 'invoice:issue'
               UNION ALL SELECT 'invoice:cancel'
               UNION ALL SELECT 'warehouse:read'
               UNION ALL SELECT 'warehouse:create'
               UNION ALL SELECT 'warehouse:update'
               UNION ALL SELECT 'warehouse:post'
             ) s
         )::text
       WHERE name = 'accountant'
      SQL

    execute <<-SQL
      UPDATE roles
         SET permissions = (
           SELECT COALESCE(jsonb_agg(DISTINCT val), '[]'::jsonb)
             FROM (
               SELECT jsonb_array_elements_text(permissions::jsonb) AS val
               UNION ALL SELECT 'warehouse:read'
             ) s
         )::text
       WHERE name IN ('viewer', 'read_only');
    SQL
  end

  def rollback
    execute "DROP TABLE IF EXISTS stock_balances"
    execute "ALTER TABLE companies ALTER COLUMN inventory_valuation_method SET DEFAULT 'FIFO'"
  end
end
