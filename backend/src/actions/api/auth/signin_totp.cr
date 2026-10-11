# POST /api/auth/signin/totp — second step after a correct password.
class Api::Auth::SignIn::Totp < ApiAction
  include Api::Auth::SkipRequireAuthToken
  include Api::Auth::JsonBody

  post "/api/auth/signin/totp" do
    body = json_body
    challenge = json_field(body, "challenge_token")
    code = json_field(body, "code")

    user = UserToken.user_from_2fa_challenge(challenge)
    unless user
      response.status_code = 401
      return json({success: false, message: "Сесията за потвърждение изтече. Влезте отново."})
    end

    email = user.email
    if message = LoginAttemptTracker.lock_message(email) || LoginAttemptTracker.lock_message("#{email}:2fa")
      AuditLogger.log(context, "login_locked", email)
      response.status_code = 429
      return json({success: false, message: message})
    end

    unless user.is_active
      response.status_code = 403
      return json({success: false, message: "Акаунтът е деактивиран. Свържете се с администратор."})
    end

    unless user.otp_enabled
      response.status_code = 401
      return json({success: false, message: "Двуфакторната защита не е включена. Влезте отново."})
    end

    if ::TwoFactor.authenticate(user, code)
      LoginAttemptTracker.record_success(email.downcase)
      LoginAttemptTracker.record_success("#{email}:2fa".downcase)
      AuditLogger.log(context, "login", email)
      token = UserToken.generate(user)
      json({
        success: true,
        token:   token,
        user:    {
          id:       user.id,
          email:    user.email,
          username: user.email.split('@').first,
        },
      })
    else
      LoginAttemptTracker.record_failure("#{email}:2fa".downcase)
      AuditLogger.log(context, "login_2fa_failed", email)
      response.status_code = 401
      json({success: false, message: "Невалиден код"})
    end
  end
end
