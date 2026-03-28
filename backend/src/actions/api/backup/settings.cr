# GET /api/backup/settings - Get S3 backup settings
class Api::Backup::Settings::Index < ApiAction
  include Api::Auth::SkipRequireAuthToken

  get "/api/backup/settings" do
    setting = SystemSetting.get("backup_s3")

    if setting
      parsed = setting.parsed_value
      # Don't return secret key for security
      json({
        success: true,
        data:    {
          endpoint:         parsed["endpoint"]?.try(&.as_s?) || "",
          bucket:           parsed["bucket"]?.try(&.as_s?) || "",
          access_key:       parsed["access_key"]?.try(&.as_s?) || "",
          has_secret_key:   !parsed["secret_key"]?.try(&.as_s?).to_s.empty?,
          region:           parsed["region"]?.try(&.as_s?) || "us-east-1",
          prefix:           parsed["prefix"]?.try(&.as_s?) || "backups/",
          retention_days:   parsed["retention_days"]?.try(&.as_i?) || 30,
          schedule_enabled: parsed["schedule_enabled"]?.try(&.as_bool?) || false,
          schedule_time:    parsed["schedule_time"]?.try(&.as_s?) || "03:00",
          schedule_days:    parsed["schedule_days"]?.try(&.as_s?) || "daily",
        },
      })
    else
      json({
        success: true,
        data:    {
          endpoint:         "",
          bucket:           "",
          access_key:       "",
          has_secret_key:   false,
          region:           "us-east-1",
          prefix:           "backups/",
          retention_days:   30,
          schedule_enabled: false,
          schedule_time:    "03:00",
          schedule_days:    "daily",
        },
      })
    end
  end
end

# POST /api/backup/settings - Save S3 backup settings
class Api::Backup::Settings::Save < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/backup/settings" do
    body = parse_json_body

    # Get existing secret key if not provided
    existing = SystemSetting.get("backup_s3")
    existing_secret = if existing
                        existing.parsed_value["secret_key"]?.try(&.as_s?) || ""
                      else
                        ""
                      end

    secret_key = body["secret_key"]?.try(&.as_s?)
    secret_key = existing_secret if secret_key.nil? || secret_key.empty?

    settings = {
      endpoint:         body["endpoint"]?.try(&.as_s?) || "",
      bucket:           body["bucket"]?.try(&.as_s?) || "",
      access_key:       body["access_key"]?.try(&.as_s?) || "",
      secret_key:       secret_key,
      region:           body["region"]?.try(&.as_s?) || "us-east-1",
      prefix:           body["prefix"]?.try(&.as_s?) || "backups/",
      retention_days:   body["retention_days"]?.try(&.as_i?) || 30,
      schedule_enabled: body["schedule_enabled"]?.try(&.as_bool?) || false,
      schedule_time:    body["schedule_time"]?.try(&.as_s?) || "03:00",
      schedule_days:    body["schedule_days"]?.try(&.as_s?) || "daily",
    }

    SystemSetting.set("backup_s3", settings, "S3 Backup Configuration")

    json({success: true, message: "Настройките са запазени"})
  rescue ex
    json({success: false, error: ex.message})
  end

  private def parse_json_body : JSON::Any
    body_str = request.body.try(&.gets_to_end) || "{}"
    JSON.parse(body_str)
  end
end
