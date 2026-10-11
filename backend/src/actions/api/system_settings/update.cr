class Api::SystemSettings::Update < ApiAction
  put "/api/system_settings/:key" do
    unless current_user?.try(&.is_super_admin)
      response.status_code = 403
      return json({success: false, message: "Само супер администратор може да променя системни настройки"})
    end
    update_setting(key)
  end

  post "/api/system_settings/:key" do
    unless current_user?.try(&.is_super_admin)
      response.status_code = 403
      return json({success: false, message: "Само супер администратор може да променя системни настройки"})
    end
    update_setting(key)
  end

  private def update_setting(key : String)
    setting = SystemSettingQuery.new.key(key).first?

    # Try to get value from JSON body first, then fall back to form params
    value_str = nil
    begin
      json_params = params.from_json
      value_str = json_params["value"]?.try(&.as_s?)
    rescue
    end
    value_str ||= params.get?(:value)

    if setting.nil?
      # Create new setting
      setting = SaveSystemSetting.create!(
        key: key,
        value: value_str || "{}",
        description: params.get?(:description)
      )
    else
      # Update existing
      SaveSystemSetting.update!(setting,
        value: value_str || setting.value,
        description: params.get?(:description) || setting.description
      )
    end

    json({
      success: true,
      data:    {
        id:          setting.id,
        key:         setting.key,
        value:       setting.parsed_value,
        description: setting.description,
      },
    })
  end
end
