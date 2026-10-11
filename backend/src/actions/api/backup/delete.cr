# DELETE /api/backup/:filename - Delete a backup from S3 (super admin + password)
class Api::Backup::Delete < ApiAction
  delete "/api/backup/:filename" do
    handle_delete
  end

  # POST variant so the password can be sent in the request body
  post "/api/backup/delete/:filename" do
    handle_delete
  end

  private def handle_delete : Lucky::Response
    user = current_user?
    unless user && user.super_admin?
      response.status_code = 403
      return json({success: false, message: "Само супер администратор може да изтрива backup"})
    end

    unless valid_backup_filename?(filename)
      response.status_code = 422
      return json({success: false, message: "Невалидно име на файл"})
    end

    body = parse_body
    password = body["password"]?.try(&.as_s?) || ""

    unless Authentic.correct_password?(user, password)
      AuditLogger.log(context, "backup_delete_denied", user.email, filename)
      response.status_code = 422
      return json({success: false, message: "Грешна парола"})
    end

    AuditLogger.log(context, "backup_delete", user.email, filename)
    result = DatabaseBackup.delete_backup(filename)

    json({
      success: result[:success],
      message: result[:message],
    })
  rescue ex
    json({success: false, message: ex.message})
  end

  private def valid_backup_filename?(name : String) : Bool
    !name.empty? && !name.includes?("/") && !name.includes?("\\") && !name.includes?("..")
  end

  private def parse_body
    body_str = request.body.try(&.gets_to_end) || "{}"
    JSON.parse(body_str)
  end
end
