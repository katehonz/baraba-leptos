class ExtendInvoiceLines::V20260912000006 < Avram::Migrator::Migration::V1
  def migrate
    execute <<-SQL
      ALTER TABLE invoice_lines
        ADD COLUMN IF NOT EXISTS product_id BIGINT REFERENCES products(id) ON DELETE SET NULL,
        ADD COLUMN IF NOT EXISTS unit_code VARCHAR(10) NOT NULL DEFAULT 'C62',
        ADD COLUMN IF NOT EXISTS vat_amount NUMERIC NOT NULL DEFAULT 0,
        ADD COLUMN IF NOT EXISTS line_total_with_vat NUMERIC NOT NULL DEFAULT 0,
        ADD COLUMN IF NOT EXISTS vat_exemption_reason_id BIGINT REFERENCES vat_exemption_reasons(id) ON DELETE SET NULL,
        ADD COLUMN IF NOT EXISTS line_number INTEGER NOT NULL DEFAULT 1,
        ADD COLUMN IF NOT EXISTS tax_type VARCHAR(16),
        ADD COLUMN IF NOT EXISTS tax_code VARCHAR(16),
        ADD COLUMN IF NOT EXISTS discount_percent NUMERIC NOT NULL DEFAULT 0,
        ADD COLUMN IF NOT EXISTS discount_amount NUMERIC NOT NULL DEFAULT 0
      SQL
    execute "CREATE INDEX IF NOT EXISTS invoice_lines_product_id_index ON invoice_lines (product_id)"
  end

  def rollback
    execute <<-SQL
      ALTER TABLE invoice_lines
        DROP COLUMN IF EXISTS discount_amount,
        DROP COLUMN IF EXISTS discount_percent,
        DROP COLUMN IF EXISTS tax_code,
        DROP COLUMN IF EXISTS tax_type,
        DROP COLUMN IF EXISTS line_number,
        DROP COLUMN IF EXISTS vat_exemption_reason_id,
        DROP COLUMN IF EXISTS line_total_with_vat,
        DROP COLUMN IF EXISTS vat_amount,
        DROP COLUMN IF EXISTS unit_code,
        DROP COLUMN IF EXISTS product_id;
    SQL
  end
end
