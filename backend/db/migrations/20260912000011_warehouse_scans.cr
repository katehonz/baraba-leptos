class WarehouseScans::V20260912000011 < Avram::Migrator::Migration::V1
  def migrate
    execute "ALTER TABLE vendor_product_mappings ADD COLUMN IF NOT EXISTS vendor_barcode VARCHAR(128)"
    execute "ALTER TABLE vendor_product_mappings ADD COLUMN IF NOT EXISTS vendor_name_normalized VARCHAR(255)"
    execute "ALTER TABLE vendor_product_mappings ADD COLUMN IF NOT EXISTS vendor_unit_code VARCHAR(10)"
    execute "ALTER TABLE vendor_product_mappings ADD COLUMN IF NOT EXISTS conversion_factor NUMERIC NOT NULL DEFAULT 1"
    execute "ALTER TABLE vendor_product_mappings ADD COLUMN IF NOT EXISTS confirmed BOOLEAN NOT NULL DEFAULT false"
    execute "ALTER TABLE vendor_product_mappings ADD COLUMN IF NOT EXISTS hit_count INTEGER NOT NULL DEFAULT 0"
    execute "ALTER TABLE vendor_product_mappings ADD COLUMN IF NOT EXISTS last_seen_at TIMESTAMPTZ"
    execute "CREATE INDEX IF NOT EXISTS vendor_product_mappings_barcode_idx ON vendor_product_mappings (company_id, counterpart_id, vendor_barcode)"
    execute "CREATE INDEX IF NOT EXISTS vendor_product_mappings_norm_idx ON vendor_product_mappings (company_id, counterpart_id, vendor_name_normalized)"

    execute <<-SQL
      CREATE TABLE IF NOT EXISTS warehouse_scans (
        id BIGSERIAL PRIMARY KEY,
        company_id BIGINT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
        direction VARCHAR(16) NOT NULL DEFAULT 'inbound',
        status VARCHAR(16) NOT NULL DEFAULT 'pending',
        original_file_name VARCHAR(255),
        s3_key TEXT,
        raw_json TEXT NOT NULL DEFAULT '{}',
        vendor_name VARCHAR(255),
        vendor_vat VARCHAR(32),
        invoice_number VARCHAR(64),
        document_date TIMESTAMPTZ,
        subtotal NUMERIC,
        vat_amount NUMERIC,
        total_amount NUMERIC,
        confidence NUMERIC,
        notes TEXT,
        counterpart_id BIGINT REFERENCES counterparts(id) ON DELETE SET NULL,
        warehouse_document_id BIGINT REFERENCES warehouse_documents(id) ON DELETE SET NULL,
        invoice_id BIGINT REFERENCES invoices(id) ON DELETE SET NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
      SQL
    execute "CREATE INDEX IF NOT EXISTS warehouse_scans_company_id_index ON warehouse_scans (company_id)"

    execute <<-SQL
      CREATE TABLE IF NOT EXISTS warehouse_scan_lines (
        id BIGSERIAL PRIMARY KEY,
        warehouse_scan_id BIGINT NOT NULL REFERENCES warehouse_scans(id) ON DELETE CASCADE,
        raw_name TEXT,
        raw_sku VARCHAR(128),
        raw_barcode VARCHAR(128),
        quantity NUMERIC NOT NULL DEFAULT 1,
        raw_unit VARCHAR(64),
        unit_code VARCHAR(10) NOT NULL DEFAULT 'C62',
        unit_price NUMERIC NOT NULL DEFAULT 0,
        vat_rate NUMERIC NOT NULL DEFAULT 20,
        mapping_status VARCHAR(16) NOT NULL DEFAULT 'unmapped',
        confidence NUMERIC,
        line_number INTEGER NOT NULL DEFAULT 1,
        product_id BIGINT REFERENCES products(id) ON DELETE SET NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
      SQL
    execute "CREATE INDEX IF NOT EXISTS warehouse_scan_lines_scan_id_index ON warehouse_scan_lines (warehouse_scan_id)"
  end

  def rollback
    execute "DROP TABLE IF EXISTS warehouse_scan_lines"
    execute "DROP TABLE IF EXISTS warehouse_scans"
    execute "ALTER TABLE vendor_product_mappings DROP COLUMN IF EXISTS vendor_barcode"
    execute "ALTER TABLE vendor_product_mappings DROP COLUMN IF EXISTS vendor_name_normalized"
    execute "ALTER TABLE vendor_product_mappings DROP COLUMN IF EXISTS vendor_unit_code"
    execute "ALTER TABLE vendor_product_mappings DROP COLUMN IF EXISTS conversion_factor"
    execute "ALTER TABLE vendor_product_mappings DROP COLUMN IF EXISTS confirmed"
    execute "ALTER TABLE vendor_product_mappings DROP COLUMN IF EXISTS hit_count"
    execute "ALTER TABLE vendor_product_mappings DROP COLUMN IF EXISTS last_seen_at"
  end
end
