class CreateSystemSettings::V20250129000002 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(SystemSetting) do
      primary_key id : Int64
      add_timestamps
      add key : String, unique: true
      add value : String, default: "{}"
      add description : String?
    end

    # Create index for key lookups
    execute "CREATE INDEX IF NOT EXISTS system_settings_key_idx ON system_settings (key);"
  end

  def rollback
    drop table_for(SystemSetting)
  end
end
