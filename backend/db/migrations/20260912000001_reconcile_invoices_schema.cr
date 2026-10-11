class ReconcileInvoicesSchema::V20260912000001 < Avram::Migrator::Migration::V1
  def migrate
    execute <<-SQL
      DO $$
      BEGIN
        IF EXISTS (
          SELECT 1 FROM information_schema.columns
          WHERE table_schema = 'public'
            AND table_name = 'invoices'
            AND column_name = 'invoice_date'
        ) THEN
          ALTER TABLE invoices RENAME COLUMN invoice_date TO issue_date;
        END IF;
      END $$;
    SQL

    execute "ALTER TABLE invoices ADD COLUMN IF NOT EXISTS document_type text NOT NULL DEFAULT 'INVOICE'"
    execute "ALTER TABLE invoices ADD COLUMN IF NOT EXISTS direction text NOT NULL DEFAULT 'sale'"
    execute "ALTER TABLE invoices ADD COLUMN IF NOT EXISTS exchange_rate numeric NOT NULL DEFAULT 1.0"
    execute "ALTER TABLE invoices ADD COLUMN IF NOT EXISTS payment_method text"
    execute "ALTER TABLE invoices ADD COLUMN IF NOT EXISTS tax_event_date timestamptz"
    execute "ALTER TABLE invoices ADD COLUMN IF NOT EXISTS vat_exemption_reason text"
    execute "ALTER TABLE invoices ADD COLUMN IF NOT EXISTS has_inventory boolean NOT NULL DEFAULT false"
    execute "ALTER TABLE invoices ADD COLUMN IF NOT EXISTS original_invoice_id bigint"
    execute "ALTER TABLE invoices ADD COLUMN IF NOT EXISTS journal_entry_id bigint"
    execute "ALTER TABLE invoices ADD COLUMN IF NOT EXISTS bank_account_id bigint"

    make_optional :invoices, :invoice_number

    execute <<-SQL
      UPDATE invoices SET lines = '[]' WHERE lines IS NULL;
    SQL
    make_required :invoices, :lines

    execute <<-SQL
      DO $$
      BEGIN
        IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'invoices_original_invoice_id_fkey') THEN
          ALTER TABLE invoices
            ADD CONSTRAINT invoices_original_invoice_id_fkey
            FOREIGN KEY (original_invoice_id) REFERENCES invoices(id) ON DELETE RESTRICT;
        END IF;
        IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'invoices_journal_entry_id_fkey') THEN
          ALTER TABLE invoices
            ADD CONSTRAINT invoices_journal_entry_id_fkey
            FOREIGN KEY (journal_entry_id) REFERENCES journal_entries(id) ON DELETE SET NULL;
        END IF;
        IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'invoices_bank_account_id_fkey') THEN
          ALTER TABLE invoices
            ADD CONSTRAINT invoices_bank_account_id_fkey
            FOREIGN KEY (bank_account_id) REFERENCES bank_accounts(id) ON DELETE SET NULL;
        END IF;
      END $$;
    SQL

    execute "CREATE INDEX IF NOT EXISTS invoices_original_invoice_id_index ON invoices (original_invoice_id)"
    execute "CREATE INDEX IF NOT EXISTS invoices_journal_entry_id_index ON invoices (journal_entry_id)"
    execute "CREATE INDEX IF NOT EXISTS invoices_bank_account_id_index ON invoices (bank_account_id)"

    drop_index :invoices, [:company_id, :invoice_number], if_exists: true

    execute <<-SQL
      CREATE UNIQUE INDEX IF NOT EXISTS invoices_company_id_invoice_number_sale_uidx
        ON invoices (company_id, invoice_number)
        WHERE invoice_number IS NOT NULL AND direction = 'sale';
    SQL
  end

  def rollback
    execute <<-SQL
      DROP INDEX IF EXISTS invoices_company_id_invoice_number_sale_uidx;
    SQL

    create_index :invoices, [:company_id, :invoice_number], unique: true

    execute <<-SQL
      UPDATE invoices SET invoice_number = 'DRAFT-' || id::text WHERE invoice_number IS NULL;
    SQL
    make_required :invoices, :invoice_number
    make_optional :invoices, :lines

    execute "ALTER TABLE invoices DROP CONSTRAINT IF EXISTS invoices_bank_account_id_fkey"
    execute "ALTER TABLE invoices DROP CONSTRAINT IF EXISTS invoices_journal_entry_id_fkey"
    execute "ALTER TABLE invoices DROP CONSTRAINT IF EXISTS invoices_original_invoice_id_fkey"
    execute "DROP INDEX IF EXISTS invoices_bank_account_id_index"
    execute "DROP INDEX IF EXISTS invoices_journal_entry_id_index"
    execute "DROP INDEX IF EXISTS invoices_original_invoice_id_index"
    execute "ALTER TABLE invoices DROP COLUMN IF EXISTS bank_account_id"
    execute "ALTER TABLE invoices DROP COLUMN IF EXISTS journal_entry_id"
    execute "ALTER TABLE invoices DROP COLUMN IF EXISTS original_invoice_id"
    execute "ALTER TABLE invoices DROP COLUMN IF EXISTS has_inventory"
    execute "ALTER TABLE invoices DROP COLUMN IF EXISTS vat_exemption_reason"
    execute "ALTER TABLE invoices DROP COLUMN IF EXISTS tax_event_date"
    execute "ALTER TABLE invoices DROP COLUMN IF EXISTS payment_method"
    execute "ALTER TABLE invoices DROP COLUMN IF EXISTS exchange_rate"
    execute "ALTER TABLE invoices DROP COLUMN IF EXISTS direction"
    execute "ALTER TABLE invoices DROP COLUMN IF EXISTS document_type"

    execute <<-SQL
      DO $$
      BEGIN
        IF EXISTS (
          SELECT 1 FROM information_schema.columns
          WHERE table_schema = 'public'
            AND table_name = 'invoices'
            AND column_name = 'issue_date'
        ) THEN
          ALTER TABLE invoices RENAME COLUMN issue_date TO invoice_date;
        END IF;
      END $$;
    SQL
  end
end
