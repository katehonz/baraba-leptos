# POST /api/backup/create - Create a new database backup
class Api::Backup::Create < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/backup/create" do
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
end
