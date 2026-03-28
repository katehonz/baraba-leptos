# POST /api/auth/resend-verification - Resend verification email
# Rate limited to prevent abuse (60 seconds between requests)
class Api::Auth::ResendVerification < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/auth/resend_verification" do
    json_params = params.from_json
    email = json_params["email"]?.try(&.as_s?) || ""

    if email.empty?
      response.status_code = 400
      return json({success: false, message: "Email е задължителен"})
    end

    user = UserQuery.new.email(email).first?

    unless user
      # Don't reveal if user exists or not (security)
      return json({
        success: true,
        message: "Ако има акаунт с този email, ще получите нов email за потвърждение.",
      })
    end

    if user.email_verified?
      return json({
        success: true,
        message: "Email вече е потвърден. Можете да влезете.",
      })
    end

    # Check rate limiting
    unless user.can_resend_verification?
      remaining = user.seconds_until_can_resend
      response.status_code = 429
      return json({
        success:           false,
        message:           "Моля, изчакайте #{remaining} секунди преди да изпратите отново.",
        seconds_remaining: remaining,
        rate_limited:      true,
      })
    end

    # Check if app_url is configured
    app_url = SystemSetting.app_url
    if app_url.nil? || app_url.empty?
      response.status_code = 500
      return json({
        success: false,
        message: "Системата не е конфигурирана правилно. Моля, свържете се с администратора.",
      })
    end

    # Generate new verification token
    new_token = Random::Secure.hex(32)
    SaveUser.update!(user,
      email_verification_token: new_token,
      verification_email_sent_at: Time.utc
    )

    # Reload user to get new token
    reloaded_user = UserQuery.new.id(user.id).first?
    user = reloaded_user.not_nil!

    # Send verification email
    verify_url = "#{app_url}/verify-email?token=#{user.email_verification_token}"
    email_sent = EmailService.send_verification_email(user, verify_url)

    if email_sent
      json({
        success:           true,
        message:           "Изпратихме нов email за потвърждение на #{email}.",
        cooldown_seconds:  User::RESEND_COOLDOWN_SECONDS,
      })
    else
      response.status_code = 500
      json({
        success: false,
        message: "Грешка при изпращане на email. Моля, опитайте отново по-късно.",
      })
    end
  end
end
