class AddLinesToInvoices::V20250126000031 < Avram::Migrator::Migration::V1
  def migrate
    execute <<-SQL
      ALTER TABLE invoices ADD COLUMN IF NOT EXISTS lines TEXT DEFAULT '[]';
    SQL
  end

  def rollback
    execute <<-SQL
      ALTER TABLE invoices DROP COLUMN IF EXISTS lines;
    SQL
  end
end
