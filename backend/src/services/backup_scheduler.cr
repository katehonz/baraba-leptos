# Background scheduler for automatic database backups
# Runs as a Crystal fiber, checks every 60 seconds if a backup should be created
class BackupScheduler
  Log = ::Log.for("backup.scheduler")

  DAY_NAMES = {"mon", "tue", "wed", "thu", "fri", "sat", "sun"}

  @@last_backup_date : String = ""
  @@running : Bool = false

  def self.start
    return if @@running
    @@running = true

    spawn do
      Log.info { "Backup scheduler started" }
      loop do
        begin
          check_and_run
        rescue ex
          Log.error { "Scheduler error: #{ex.message}" }
        end
        sleep 60.seconds
      end
    end
  end

  private def self.check_and_run
    return unless DatabaseBackup.configured?

    settings = DatabaseBackup.s3_settings
    return unless settings["schedule_enabled"] == "true"

    schedule_time = settings["schedule_time"]
    schedule_days = settings["schedule_days"]

    now = Time.utc
    current_time = now.to_s("%H:%M")
    current_date = now.to_s("%Y-%m-%d")

    # Already ran today
    return if @@last_backup_date == current_date

    # Check time match (within the same minute)
    return unless current_time == schedule_time

    # Check day match
    return unless day_matches?(now, schedule_days)

    # Run backup
    @@last_backup_date = current_date
    Log.info { "Scheduled backup starting (#{schedule_days} @ #{schedule_time})" }

    result = DatabaseBackup.create_backup
    if result[:success]
      Log.info { "Scheduled backup completed: #{result[:message]}" }
    else
      Log.error { "Scheduled backup failed: #{result[:message]}" }
    end
  end

  private def self.day_matches?(time : Time, schedule_days : String) : Bool
    return true if schedule_days == "daily"

    current_day = time.day_of_week
    day_str = case current_day
              when .monday?    then "mon"
              when .tuesday?   then "tue"
              when .wednesday? then "wed"
              when .thursday?  then "thu"
              when .friday?    then "fri"
              when .saturday?  then "sat"
              when .sunday?    then "sun"
              else                  ""
              end

    days = schedule_days.split(",").map(&.strip.downcase)
    days.includes?(day_str)
  end
end
