class LinkWarehousesToLocations::V20260912000003 < Avram::Migrator::Migration::V1
  def migrate
    execute "ALTER TABLE company_locations ADD COLUMN IF NOT EXISTS holds_stock BOOLEAN NOT NULL DEFAULT false"
    execute "ALTER TABLE company_locations ADD COLUMN IF NOT EXISTS branch_code VARCHAR(8)"
    execute "ALTER TABLE company_locations ADD COLUMN IF NOT EXISTS saft_location_code VARCHAR(32)"
    execute "UPDATE company_locations SET holds_stock = true WHERE location_type IN ('WAREHOUSE', 'STORE')"
    execute "UPDATE company_locations SET saft_location_code = id::text WHERE saft_location_code IS NULL"
    execute "ALTER TABLE warehouses ADD COLUMN IF NOT EXISTS company_location_id BIGINT"
    execute <<-SQL
      INSERT INTO company_locations (
        name, location_type, country, is_main, is_active, holds_stock, company_id, created_at, updated_at, saft_location_code
      )
      SELECT w.name, 'WAREHOUSE', 'BG', false, w.is_active, true, w.company_id, NOW(), NOW(), w.id::text
        FROM warehouses w
       WHERE w.company_location_id IS NULL
      SQL
    execute <<-SQL
      UPDATE warehouses w
         SET company_location_id = loc.id
        FROM company_locations loc
       WHERE w.company_location_id IS NULL
         AND loc.company_id = w.company_id
         AND loc.name = w.name
         AND loc.location_type = 'WAREHOUSE'
         AND loc.holds_stock = true
      SQL
    execute <<-SQL
      INSERT INTO warehouses (name, is_active, company_id, company_location_id, created_at, updated_at)
      SELECT loc.name, loc.is_active, loc.company_id, loc.id, NOW(), NOW()
        FROM company_locations loc
       WHERE loc.holds_stock = true
         AND NOT EXISTS (SELECT 1 FROM warehouses w WHERE w.company_location_id = loc.id)
      SQL
    execute <<-SQL
      UPDATE warehouses SET company_location_id = (
        SELECT loc.id FROM company_locations loc
         WHERE loc.company_id = warehouses.company_id
         ORDER BY loc.id LIMIT 1
      ) WHERE company_location_id IS NULL
      SQL
    execute "ALTER TABLE warehouses ALTER COLUMN company_location_id SET NOT NULL"
    execute "CREATE UNIQUE INDEX IF NOT EXISTS warehouses_company_location_id_index ON warehouses (company_location_id)"
  end

  def rollback
    execute "DROP INDEX IF EXISTS warehouses_company_location_id_index"
    execute "ALTER TABLE warehouses DROP COLUMN IF EXISTS company_location_id"
    execute "ALTER TABLE company_locations DROP COLUMN IF EXISTS holds_stock"
    execute "ALTER TABLE company_locations DROP COLUMN IF EXISTS branch_code"
    execute "ALTER TABLE company_locations DROP COLUMN IF EXISTS saft_location_code"
  end
end
