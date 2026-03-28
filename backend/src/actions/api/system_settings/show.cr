class Api::SystemSettings::Show < ApiAction
  get "/api/system_settings/:key" do
    unless current_user?.try(&.is_super_admin)
      response.status_code = 403
      return json({success: false, message: "Само супер администратор може да вижда системни настройки"})
    end

    setting = SystemSettingQuery.new.key(key).first?

    if setting
      json({
        success: true,
        data:    {
          id:          setting.id,
          key:         setting.key,
          value:       setting.parsed_value,
          description: setting.description,
        },
      })
    else
      json({
        success: false,
        message: "Setting not found",
      })
    end
  end
end
