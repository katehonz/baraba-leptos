class InvoiceIssuedLocationNap::V20260912000007 < Avram::Migrator::Migration::V1
  def migrate
    execute <<-SQL
      ALTER TABLE invoices
        ADD COLUMN IF NOT EXISTS location_id BIGINT REFERENCES company_locations(id) ON DELETE SET NULL,
        ADD COLUMN IF NOT EXISTS nap_document_type VARCHAR(2),
        ADD COLUMN IF NOT EXISTS uncl1001_code VARCHAR(10),
        ADD COLUMN IF NOT EXISTS issued_at TIMESTAMPTZ,
        ADD COLUMN IF NOT EXISTS cancelled_at TIMESTAMPTZ,
        ADD COLUMN IF NOT EXISTS cancellation_reason TEXT
      SQL
    execute "CREATE INDEX IF NOT EXISTS invoices_location_id_index ON invoices (location_id)"
    execute "CREATE INDEX IF NOT EXISTS invoices_company_direction_issue_date_index ON invoices (company_id, direction, issue_date)"
    execute "DROP INDEX IF EXISTS invoices_company_id_invoice_number_sale_uidx"
    execute <<-SQL
      CREATE UNIQUE INDEX invoices_sales_sequence_number_uidx
        ON invoices (company_id, sequence_id, invoice_number)
        WHERE sequence_id IS NOT NULL
          AND invoice_number IS NOT NULL
          AND status IN ('issued','posted','cancelled')
      SQL
    execute <<-SQL
      CREATE UNIQUE INDEX invoices_purchase_number_uidx
        ON invoices (company_id, counterpart_id, invoice_number, issue_date)
        WHERE direction = 'purchase'
          AND status IN ('issued','posted')
      SQL
  end

  def rollback
    execute "DROP INDEX IF EXISTS invoices_purchase_number_uidx"
    execute "DROP INDEX IF EXISTS invoices_sales_sequence_number_uidx"
    execute <<-SQL
      CREATE UNIQUE INDEX IF NOT EXISTS invoices_company_id_invoice_number_sale_uidx
        ON invoices (company_id, invoice_number)
        WHERE invoice_number IS NOT NULL AND direction = 'sale'
      SQL
    execute "ALTER TABLE invoices DROP COLUMN IF EXISTS cancellation_reason"
    execute "ALTER TABLE invoices DROP COLUMN IF EXISTS cancelled_at"
    execute "ALTER TABLE invoices DROP COLUMN IF EXISTS issued_at"
    execute "ALTER TABLE invoices DROP COLUMN IF EXISTS uncl1001_code"
    execute "ALTER TABLE invoices DROP COLUMN IF EXISTS nap_document_type"
    execute "ALTER TABLE invoices DROP COLUMN IF EXISTS location_id"
  end
end
