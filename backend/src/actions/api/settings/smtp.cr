class Api::Settings::Smtp < ApiAction
  get "/api/settings/smtp" do
    smtp = SystemSetting.smtp_settings

    json({
      success: true,
      data:    {
        smtp_host:       SystemSetting.smtp_host,
        smtp_port:       SystemSetting.smtp_port,
        smtp_username:   SystemSetting.smtp_username,
        smtp_from_email: SystemSetting.smtp_from_email,
        smtp_from_name:  SystemSetting.smtp_from_name,
        smtp_use_tls:    SystemSetting.smtp_use_tls?,
        smtp_enabled:    SystemSetting.smtp_enabled?,
      },
    })
  end
end

class Api::Settings::SmtpUpdate < ApiAction
  post "/api/settings/smtp" do
    # Parse JSON body
    json_params = params.from_json

    smtp_data = {
      "host"       => json_params["smtp_host"]?.try(&.as_s?) || "",
      "port"       => json_params["smtp_port"]?.try(&.as_i?) || 587,
      "username"   => json_params["smtp_username"]?.try(&.as_s?) || "",
      "password"   => json_params["smtp_password"]?.try(&.as_s?) || "",
      "from_email" => json_params["smtp_from_email"]?.try(&.as_s?) || "",
      "from_name"  => json_params["smtp_from_name"]?.try(&.as_s?) || "",
      "use_tls"    => json_params["smtp_use_tls"]?.try(&.as_bool?) || false,
      "enabled"    => json_params["smtp_enabled"]?.try(&.as_bool?) || false,
    }

    # Get existing setting or create new one
    setting = SystemSetting.get("smtp")
    if setting
      # Merge with existing to preserve password if not provided
      existing = setting.parsed_value.as_h? || {} of String => JSON::Any
      if smtp_data["password"].to_s.empty? && existing["password"]?
        smtp_data["password"] = existing["password"].as_s? || ""
      end
      SaveSystemSetting.update!(setting, value: smtp_data.to_json)
    else
      SaveSystemSetting.create!(key: "smtp", value: smtp_data.to_json, description: "SMTP email настройки")
    end

    json({
      success: true,
      message: "SMTP настройките са запазени",
    })
  end
end

class Api::Settings::SmtpTest < ApiAction
  post "/api/settings/smtp/test" do
    # Parse JSON body
    json_params = params.from_json

    # Get test email recipient
    test_email = json_params["test_email"]?.try(&.as_s?) || ""

    if test_email.empty?
      response.status_code = 422
      return json({
        success: false,
        message: "Моля въведете email за тест",
      })
    end

    # Get SMTP settings from params (for testing before save)
    host = json_params["smtp_host"]?.try(&.as_s?) || ""
    port = json_params["smtp_port"]?.try(&.as_i?) || 587
    username = json_params["smtp_username"]?.try(&.as_s?) || ""
    password = json_params["smtp_password"]?.try(&.as_s?) || ""
    from_email = json_params["smtp_from_email"]?.try(&.as_s?) || ""
    from_name = json_params["smtp_from_name"]?.try(&.as_s?) || ""
    use_tls = json_params["smtp_use_tls"]?.try(&.as_bool?) || false

    # If password is empty, try to get from saved settings
    if password.empty?
      password = SystemSetting.smtp_password || ""
    end

    if host.empty? || username.empty?
      response.status_code = 422
      return json({
        success: false,
        message: "SMTP хост и потребител са задължителни",
      })
    end

    if password.empty?
      response.status_code = 422
      return json({
        success: false,
        message: "SMTP парола е задължителна",
      })
    end

    # Send test email
    result = EmailService.send_test_email_with_settings(
      to: test_email,
      host: host,
      port: port,
      username: username,
      password: password,
      from_email: from_email,
      from_name: from_name,
      use_tls: use_tls
    )

    json({
      success: result[:success],
      message: result[:message],
    })
  end
end
