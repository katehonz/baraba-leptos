# POST /api/backup/test - Test S3 connection
class Api::Backup::Test < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/backup/test" do
    result = DatabaseBackup.test_connection

    json({
      success: result[:success],
      message: result[:message],
    })
  rescue ex
    json({success: false, message: ex.message})
  end
end
