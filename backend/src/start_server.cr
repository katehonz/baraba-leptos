require "./app"

Habitat.raise_if_missing_settings!

# Run migrations in all environments
puts "Running database migrations..."
Avram::Migrator::Runner.new.run_pending_migrations
puts "Migrations complete."

if LuckyEnv.development?
  Avram::SchemaEnforcer.ensure_correct_column_mappings!
end

BackupScheduler.start

app_server = AppServer.new
puts "Listening on http://#{app_server.host}:#{app_server.port}"

Signal::INT.trap do
  app_server.close
end

app_server.listen
