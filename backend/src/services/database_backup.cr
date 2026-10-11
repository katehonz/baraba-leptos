# Service for database backup to S3-compatible storage
# Архивиране на базата данни в S3 съвместимо хранилище
# Uses Python script with boto3 for S3 operations (compatible with MinIO, DigitalOcean Spaces, Backblaze B2, etc.)
class DatabaseBackup
  Log = ::Log.for("backup")

  PYTHON_SCRIPT = "/app/scripts/s3_backup.py"

  # Get S3 settings from system settings
  def self.s3_settings : Hash(String, String)
    setting = SystemSetting.get("backup_s3")
    return {} of String => String unless setting

    parsed = setting.parsed_value
    {
      "endpoint"         => parsed["endpoint"]?.try(&.as_s?) || "",
      "bucket"           => parsed["bucket"]?.try(&.as_s?) || "",
      "access_key"       => parsed["access_key"]?.try(&.as_s?) || "",
      "secret_key"       => parsed["secret_key"]?.try(&.as_s?) || "",
      "region"           => parsed["region"]?.try(&.as_s?) || "us-east-1",
      "prefix"           => parsed["prefix"]?.try(&.as_s?) || "backups/",
      "retention_days"   => (parsed["retention_days"]?.try(&.as_i?) || 30).to_s,
      "schedule_enabled" => (parsed["schedule_enabled"]?.try(&.as_bool?) || false).to_s,
      "schedule_time"    => parsed["schedule_time"]?.try(&.as_s?) || "03:00",
      "schedule_days"    => parsed["schedule_days"]?.try(&.as_s?) || "daily",
    }
  end

  # Check if S3 is configured
  def self.configured? : Bool
    settings = s3_settings
    !settings["bucket"]?.to_s.empty? && !settings["access_key"]?.to_s.empty? && !settings["secret_key"]?.to_s.empty?
  end

  # Create database backup and upload to S3
  def self.create_backup : NamedTuple(success: Bool, message: String, filename: String?)
    unless configured?
      return {success: false, message: "S3 не е конфигуриран", filename: nil}
    end

    settings = s3_settings
    timestamp = Time.utc.to_s("%Y%m%d_%H%M%S")
    filename = "baraba_backup_#{timestamp}.dump"
    temp_path = "/tmp/#{filename}"

    begin
      # Get database URL from environment
      db_url = ENV["DATABASE_URL"]? || "postgres://localhost/baraba_development"

      # Run pg_dump with custom format (most efficient for restore)
      Log.info { "Starting backup: #{filename}" }

      stdout = IO::Memory.new
      stderr = IO::Memory.new

      status = Process.run(
        "pg_dump",
        ["--no-owner", "--no-acl", "-Fc", "-f", temp_path, db_url],
        output: stdout,
        error: stderr
      )

      unless status.success?
        error_msg = stderr.to_s
        Log.error { "pg_dump failed: #{error_msg}" }
        return {success: false, message: "pg_dump грешка: #{error_msg}", filename: nil}
      end

      unless File.exists?(temp_path) && File.size(temp_path) > 0
        return {success: false, message: "pg_dump не създаде файл", filename: nil}
      end

      file_size = File.size(temp_path)
      Log.info { "Backup created: #{filename}, size: #{format_size(file_size)}" }

      # Upload to S3
      s3_key = "#{settings["prefix"]}#{filename}"
      upload_result = upload_to_s3(temp_path, s3_key, settings)

      # Clean up temp file
      File.delete(temp_path) if File.exists?(temp_path)

      if upload_result[:success]
        Log.info { "Backup uploaded to S3: #{s3_key}" }
        # Clean up old backups based on retention policy
        spawn { cleanup_old_backups }
        {success: true, message: "Backup създаден: #{filename} (#{format_size(file_size)})", filename: filename}
      else
        {success: false, message: upload_result[:message], filename: nil}
      end
    rescue ex
      Log.error { "Backup error: #{ex.message}" }
      File.delete(temp_path) if File.exists?(temp_path)
      {success: false, message: "Грешка: #{ex.message}", filename: nil}
    end
  end

  # Upload file to S3 using Python script
  private def self.upload_to_s3(local_path : String, s3_key : String, settings : Hash(String, String)) : NamedTuple(success: Bool, message: String)
    args = build_python_args("upload", settings)
    args << "--local-path" << local_path
    args << "--s3-key" << s3_key

    result = run_python_script(args)

    if result["success"]?.try(&.as_bool?) == true
      {success: true, message: "OK"}
    else
      {success: false, message: result["message"]?.try(&.as_s?) || "S3 upload грешка"}
    end
  end

  # List backups from S3
  def self.list_backups : Array(Hash(String, String | Int64))
    unless configured?
      return [] of Hash(String, String | Int64)
    end

    settings = s3_settings
    backups = [] of Hash(String, String | Int64)

    begin
      args = build_python_args("list", settings)
      result = run_python_script(args)

      if result["success"]?.try(&.as_bool?) == true
        data = result["data"]?.try(&.as_a?) || [] of JSON::Any
        data.each do |item|
          backups << {
            "key"           => item["key"]?.try(&.as_s?) || "",
            "filename"      => item["filename"]?.try(&.as_s?) || "",
            "size"          => (item["size"]?.try(&.as_i64?) || 0_i64),
            "last_modified" => item["last_modified"]?.try(&.as_s?) || "",
          }
        end
      end
    rescue ex
      Log.error { "S3 list error: #{ex.message}" }
    end

    backups
  end

  # Test S3 connection
  def self.test_connection : NamedTuple(success: Bool, message: String)
    unless configured?
      return {success: false, message: "S3 не е конфигуриран. Моля, попълнете настройките."}
    end

    # Check if python3 is available
    python_check = Process.run("which", ["python3"], output: Process::Redirect::Pipe)
    unless python_check.success?
      return {success: false, message: "Python3 не е инсталиран."}
    end

    # Check if boto3 is installed
    boto_check_stdout = IO::Memory.new
    boto_check_stderr = IO::Memory.new
    boto_check = Process.run("python3", ["-c", "import boto3"], output: boto_check_stdout, error: boto_check_stderr)
    unless boto_check.success?
      return {success: false, message: "boto3 не е инсталиран. Инсталирайте с: pip install boto3"}
    end

    settings = s3_settings
    args = build_python_args("test", settings)
    result = run_python_script(args)

    if result["success"]?.try(&.as_bool?) == true
      {success: true, message: result["message"]?.try(&.as_s?) || "Връзката е успешна!"}
    else
      {success: false, message: result["message"]?.try(&.as_s?) || "Грешка при тест"}
    end
  end

  # Delete a backup from S3
  def self.delete_backup(filename : String) : NamedTuple(success: Bool, message: String)
    unless configured?
      return {success: false, message: "S3 не е конфигуриран"}
    end

    settings = s3_settings
    s3_key = "#{settings["prefix"]}#{filename}"

    args = build_python_args("delete", settings)
    args << "--s3-key" << s3_key

    result = run_python_script(args)

    if result["success"]?.try(&.as_bool?) == true
      {success: true, message: "Backup изтрит: #{filename}"}
    else
      {success: false, message: result["message"]?.try(&.as_s?) || "Грешка при изтриване"}
    end
  end

  # Restore database from S3 backup
  def self.restore_backup(filename : String) : NamedTuple(success: Bool, message: String)
    unless configured?
      return {success: false, message: "S3 не е конфигуриран"}
    end

    settings = s3_settings
    s3_key = "#{settings["prefix"]}#{filename}"
    temp_path = "/tmp/#{filename}"

    begin
      # Download from S3
      Log.info { "Downloading backup: #{filename}" }

      args = build_python_args("download", settings)
      args << "--s3-key" << s3_key
      args << "--local-path" << temp_path

      result = run_python_script(args)

      unless result["success"]?.try(&.as_bool?) == true
        return {success: false, message: result["message"]?.try(&.as_s?) || "S3 download грешка"}
      end

      unless File.exists?(temp_path) && File.size(temp_path) > 0
        return {success: false, message: "Файлът не е свален от S3"}
      end

      file_size = File.size(temp_path)
      Log.info { "Downloaded: #{filename}, size: #{format_size(file_size)}" }

      # Restore with pg_restore
      db_url = ENV["DATABASE_URL"]? || "postgres://localhost/baraba_development"

      stdout = IO::Memory.new
      stderr = IO::Memory.new

      status = Process.run(
        "pg_restore",
        ["--clean", "--if-exists", "--no-owner", "--no-acl", "-d", db_url, temp_path],
        output: stdout,
        error: stderr
      )

      # pg_restore returns non-zero for warnings too, check stderr for real errors
      error_output = stderr.to_s
      if !status.success? && error_output.includes?("FATAL")
        Log.error { "pg_restore failed: #{error_output}" }
        File.delete(temp_path) if File.exists?(temp_path)
        return {success: false, message: "pg_restore грешка: #{error_output.lines.first? || error_output}"}
      end

      File.delete(temp_path) if File.exists?(temp_path)
      Log.info { "Restore completed: #{filename}" }
      {success: true, message: "Базата е възстановена от: #{filename}"}
    rescue ex
      Log.error { "Restore error: #{ex.message}" }
      File.delete(temp_path) if File.exists?(temp_path)
      {success: false, message: "Грешка: #{ex.message}"}
    end
  end

  # Delete backups older than retention_days
  def self.cleanup_old_backups
    settings = s3_settings
    retention_days = settings["retention_days"].to_i rescue 30
    return if retention_days <= 0

    cutoff = Time.utc - retention_days.days
    backups = list_backups

    backups.each do |backup|
      last_modified = backup["last_modified"].to_s
      next if last_modified.empty?

      begin
        backup_time = Time.parse(last_modified, "%Y-%m-%d %H:%M:%S", Time::Location::UTC)
        if backup_time < cutoff
          filename = backup["filename"].to_s
          Log.info { "Retention cleanup: deleting #{filename} (#{last_modified})" }
          delete_backup(filename)
        end
      rescue ex
        Log.error { "Retention cleanup error: #{ex.message}" }
      end
    end
  rescue ex
    Log.error { "Cleanup error: #{ex.message}" }
  end

  # Build common Python script arguments
  private def self.build_python_args(action : String, settings : Hash(String, String)) : Array(String)
    args = [PYTHON_SCRIPT, action]
    args << "--bucket" << settings["bucket"]
    args << "--access-key" << settings["access_key"]
    args << "--secret-key" << settings["secret_key"]
    args << "--region" << settings["region"]
    args << "--prefix" << settings["prefix"]

    unless settings["endpoint"].empty?
      args << "--endpoint" << settings["endpoint"]
    end

    args
  end

  # Run Python script and parse JSON result
  private def self.run_python_script(args : Array(String)) : JSON::Any
    stdout = IO::Memory.new
    stderr = IO::Memory.new

    status = Process.run("python3", args, output: stdout, error: stderr)

    output = stdout.to_s.strip

    if output.empty?
      err = JSON.build do |json|
        json.object do
          json.field "success", false
          json.field "message", "Празен отговор от скрипта: #{stderr.to_s}"
        end
      end
      return JSON.parse(err)
    end

    # Take only the last JSON line in case of warnings printed to stdout
    lines = output.split('\n')
    json_line = lines.reverse.find { |l| l.starts_with?('{') } || output
    JSON.parse(json_line)
  rescue ex
    err = JSON.build do |json|
      json.object do
        json.field "success", false
        json.field "message", "JSON parse грешка: #{ex.message}"
      end
    end
    JSON.parse(err)
  end

  # Format file size for display
  def self.format_size(bytes : Int64) : String
    if bytes < 1024
      "#{bytes} B"
    elsif bytes < 1024 * 1024
      "#{(bytes / 1024.0).round(1)} KB"
    elsif bytes < 1024 * 1024 * 1024
      "#{(bytes / (1024.0 * 1024)).round(1)} MB"
    else
      "#{(bytes / (1024.0 * 1024 * 1024)).round(2)} GB"
    end
  end
end
