class AddLinesToJournalEntries::V20250126000028 < Avram::Migrator::Migration::V1
  def migrate
    execute <<-SQL
      ALTER TABLE journal_entries ADD COLUMN IF NOT EXISTS lines TEXT DEFAULT '[]';
    SQL
  end

  def rollback
    execute <<-SQL
      ALTER TABLE journal_entries DROP COLUMN IF EXISTS lines;
    SQL
  end
end
