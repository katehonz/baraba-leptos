# DELETE /api/backup/:filename - Delete a backup from S3
class Api::Backup::Delete < ApiAction
  include Api::Auth::SkipRequireAuthToken

  delete "/api/backup/:filename" do
    result = DatabaseBackup.delete_backup(filename)

    json({
      success: result[:success],
      message: result[:message],
    })
  rescue ex
    json({success: false, message: ex.message})
  end
end
