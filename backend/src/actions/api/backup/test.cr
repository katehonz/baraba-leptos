# POST /api/backup/test - Test S3 connection (super admin only)
class Api::Backup::Test < ApiAction
  post "/api/backup/test" do
    user = current_user?
    unless user && user.super_admin?
      response.status_code = 403
      return json({success: false, message: "Само супер администратор може да тества S3 връзката"})
    end

    result = DatabaseBackup.test_connection

    json({
      success: result[:success],
      message: result[:message],
    })
  rescue ex
    json({success: false, message: ex.message})
  end
end
