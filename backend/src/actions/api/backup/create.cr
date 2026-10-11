# POST /api/backup/create - Create a new database backup
# Изисква автентикация, супер администратор и потвърждение с парола
class Api::Backup::Create < ApiAction
  post "/api/backup/create" do
    user = current_user?
    unless user && user.super_admin?
      response.status_code = 403
      return json({success: false, message: "Само супер администратор може да създава backup"})
    end

    body = parse_body
    password = body["password"]?.try(&.as_s?) || ""

    unless Authentic.correct_password?(user, password)
      AuditLogger.log(context, "backup_create_denied", user.email, "Грешна парола")
      response.status_code = 422
      return json({success: false, message: "Грешна парола"})
    end

    AuditLogger.log(context, "backup_create", user.email)
    result = DatabaseBackup.create_backup

    if result[:success]
      json({
        success:  true,
        message:  result[:message],
        filename: result[:filename],
      })
    else
      json({
        success: false,
        error:   result[:message],
      })
    end
  rescue ex
    json({success: false, error: ex.message})
  end

  private def parse_body
    body_str = request.body.try(&.gets_to_end) || "{}"
    JSON.parse(body_str)
  end
end
