class Api::SystemSettings::Index < ApiAction
  get "/api/system_settings" do
    unless current_user?.try(&.is_super_admin)
      response.status_code = 403
      return json({success: false, message: "Само супер администратор може да вижда системни настройки"})
    end

    settings = SystemSettingQuery.new

    json({
      success: true,
      data:    settings.map { |s| {
        id:          s.id,
        key:         s.key,
        value:       s.parsed_value,
        description: s.description,
      } },
    })
  end
end
