# POST /api/backup/restore - Restore database from S3 backup
# Изисква автентикация, супер администратор и потвърждение с парола
class Api::Backup::Restore < ApiAction
  post "/api/backup/restore" do
    user = current_user?
    unless user && user.super_admin?
      response.status_code = 403
      return json({success: false, message: "Само супер администратор може да възстановява backup"})
    end

    body = parse_body

    filename = body["filename"]?.try(&.as_s?) || ""
    password = body["password"]?.try(&.as_s?) || ""

    if filename.empty?
      json({success: false, error: "Не е посочен файл"})
    elsif filename.includes?("/") || filename.includes?("\\") || filename.includes?("..")
      response.status_code = 422
      json({success: false, error: "Невалидно име на файл"})
    elsif !Authentic.correct_password?(user, password)
      AuditLogger.log(context, "backup_restore_denied", user.email, filename)
      response.status_code = 422
      json({success: false, message: "Грешна парола"})
    else
      AuditLogger.log(context, "backup_restore", user.email, filename)
      result = DatabaseBackup.restore_backup(filename)

      if result[:success]
        json({
          success: true,
          message: result[:message],
        })
      else
        json({
          success: false,
          error:   result[:message],
        })
      end
    end
  rescue ex
    json({success: false, error: ex.message})
  end

  private def parse_body
    body_str = request.body.try(&.gets_to_end) || "{}"
    JSON.parse(body_str)
  end
end
