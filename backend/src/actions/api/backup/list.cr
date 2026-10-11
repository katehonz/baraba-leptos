# GET /api/backup/list - List all backups from S3 (super admin only)
class Api::Backup::List < ApiAction
  get "/api/backup/list" do
    user = current_user?
    unless user && user.super_admin?
      response.status_code = 403
      return json({success: false, message: "Само супер администратор може да вижда backup списъка"})
    end

    backups = DatabaseBackup.list_backups

    json({
      success: true,
      data:    backups.map { |b|
        size_bytes = b["size"].as(Int64)
        {
          key:           b["key"].as(String),
          filename:      b["filename"].as(String),
          size:          format_size(size_bytes),
          size_bytes:    size_bytes,
          last_modified: b["last_modified"].as(String),
        }
      },
    })
  rescue ex
    json({success: false, error: ex.message, data: [] of String})
  end

  private def format_size(bytes : Int64) : String
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
