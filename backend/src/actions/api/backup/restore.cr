# POST /api/backup/restore - Restore database from S3 backup
class Api::Backup::Restore < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/backup/restore" do
    body = parse_body

    filename = body["filename"]?.try(&.as_s?) || ""
    if filename.empty?
      json({success: false, error: "Не е посочен файл"})
    else
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
