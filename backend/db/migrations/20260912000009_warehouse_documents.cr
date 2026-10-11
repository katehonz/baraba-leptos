class WarehouseDocuments::V20260912000009 < Avram::Migrator::Migration::V1
  def migrate
    execute <<-SQL
      CREATE TABLE warehouse_documents (
        id BIGSERIAL PRIMARY KEY,
        company_id BIGINT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
        document_type VARCHAR(32) NOT NULL,
        document_number VARCHAR(20),
        sequence_id BIGINT REFERENCES document_number_sequences(id) ON DELETE SET NULL,
        issue_date TIMESTAMPTZ NOT NULL,
        status VARCHAR(16) NOT NULL DEFAULT 'draft',
        from_location_id BIGINT REFERENCES company_locations(id) ON DELETE RESTRICT,
        to_location_id BIGINT REFERENCES company_locations(id) ON DELETE RESTRICT,
        notes TEXT,
        journal_entry_id BIGINT REFERENCES journal_entries(id) ON DELETE SET NULL,
        source_id VARCHAR(64),
        movement_type VARCHAR(8),
        invoice_id BIGINT REFERENCES invoices(id) ON DELETE SET NULL,
        original_warehouse_document_id BIGINT REFERENCES warehouse_documents(id) ON DELETE RESTRICT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
      SQL
    execute "CREATE UNIQUE INDEX warehouse_documents_invoice_id_uidx ON warehouse_documents (invoice_id) WHERE invoice_id IS NOT NULL"
    execute <<-SQL
      CREATE UNIQUE INDEX warehouse_documents_number_uidx
        ON warehouse_documents (company_id, sequence_id, document_number)
        WHERE status IN ('posted','cancelled') AND document_number IS NOT NULL
      SQL
    execute <<-SQL
      CREATE TABLE warehouse_document_lines (
        id BIGSERIAL PRIMARY KEY,
        warehouse_document_id BIGINT NOT NULL REFERENCES warehouse_documents(id) ON DELETE CASCADE,
        product_id BIGINT NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
        quantity NUMERIC NOT NULL,
        unit_code VARCHAR(10) NOT NULL DEFAULT 'C62',
        unit_price NUMERIC NOT NULL DEFAULT 0,
        line_total NUMERIC NOT NULL DEFAULT 0,
        line_number INTEGER NOT NULL DEFAULT 1,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
      SQL
    execute "ALTER TABLE stock_transactions ADD COLUMN IF NOT EXISTS warehouse_document_id BIGINT REFERENCES warehouse_documents(id) ON DELETE SET NULL"
    execute "CREATE INDEX IF NOT EXISTS stock_transactions_warehouse_document_id_index ON stock_transactions (warehouse_document_id)"
  end

  def rollback
    execute "ALTER TABLE stock_transactions DROP COLUMN IF EXISTS warehouse_document_id"
    execute "DROP TABLE IF EXISTS warehouse_document_lines"
    execute "DROP TABLE IF EXISTS warehouse_documents"
  end
end
