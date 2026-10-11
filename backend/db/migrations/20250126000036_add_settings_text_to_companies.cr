class AddSettingsToCompanies::V20250126000034 < Avram::Migrator::Migration::V1
  def migrate
    execute <<-SQL
      ALTER TABLE companies ADD COLUMN IF NOT EXISTS settings TEXT DEFAULT '{}';
    SQL
  end

  def rollback
    execute <<-SQL
      ALTER TABLE companies DROP COLUMN IF EXISTS settings;
    SQL
  end
end
