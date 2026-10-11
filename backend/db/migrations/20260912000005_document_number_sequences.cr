class DocumentNumberSequences::V20260912000005 < Avram::Migrator::Migration::V1
  def migrate
    execute <<-SQL
      CREATE TABLE document_number_sequences (
        id BIGSERIAL PRIMARY KEY,
        company_id BIGINT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
        code VARCHAR(64) NOT NULL,
        name VARCHAR(255) NOT NULL,
        current_number BIGINT NOT NULL DEFAULT 0,
        padding INTEGER NOT NULL DEFAULT 10,
        is_tax BOOLEAN NOT NULL DEFAULT false,
        reset_policy VARCHAR(16) NOT NULL DEFAULT 'never',
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        UNIQUE (company_id, code)
      )
      SQL
    execute <<-SQL
      CREATE TABLE document_sequence_bindings (
        id BIGSERIAL PRIMARY KEY,
        company_id BIGINT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
        family VARCHAR(32) NOT NULL,
        document_type VARCHAR(64) NOT NULL,
        document_number_sequence_id BIGINT NOT NULL REFERENCES document_number_sequences(id) ON DELETE CASCADE,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        UNIQUE (company_id, family, document_type)
      )
      SQL
    execute "ALTER TABLE invoices ADD COLUMN IF NOT EXISTS sequence_id BIGINT REFERENCES document_number_sequences(id) ON DELETE SET NULL"
    execute "CREATE INDEX IF NOT EXISTS invoices_sequence_id_index ON invoices (sequence_id)"
  end

  def rollback
    execute "ALTER TABLE invoices DROP COLUMN IF EXISTS sequence_id"
    execute "DROP TABLE IF EXISTS document_sequence_bindings"
    execute "DROP TABLE IF EXISTS document_number_sequences"
  end
end
