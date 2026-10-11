# POST /api/auth/two_factor/setup
class Api::Auth::TwoFactor::Setup < ApiAction
  include Api::Auth::JsonBody

  post "/api/auth/two_factor/setup" do
    user = current_user?
    unless user
      response.status_code = 401
      return json({success: false, message: "Не сте влезли в системата"})
    end

    if message = LoginAttemptTracker.lock_message(user.email)
      AuditLogger.log(context, "login_locked", user.email)
      response.status_code = 429
      return json({success: false, message: message})
    end

    if user.otp_enabled
      response.status_code = 422
      return json({success: false, message: "Двуфакторната защита вече е включена."})
    end

    password = json_field(json_body, "password")
    unless Authentic.correct_password?(user, password)
      LoginAttemptTracker.record_failure(user.email.downcase)
      AuditLogger.log(context, "two_factor_setup_failed", user.email)
      response.status_code = 422
      return json({success: false, message: "Паролата е грешна"})
    end

    enrollment = ::TwoFactor.begin_enrollment(user)
    AuditLogger.log(context, "two_factor_setup_started", user.email)
    json({
      success:         true,
      secret:          enrollment.secret,
      otpauth_uri:     enrollment.uri,
      qr_image:        enrollment.qr_image,
      recovery_codes:  enrollment.codes,
    })
  end
end

# POST /api/auth/two_factor/confirm
class Api::Auth::TwoFactor::Confirm < ApiAction
  include Api::Auth::JsonBody

  post "/api/auth/two_factor/confirm" do
    user = current_user?
    unless user
      response.status_code = 401
      return json({success: false, message: "Не сте влезли в системата"})
    end

    if user.otp_enabled
      return json({success: true, message: "Двуфакторната защита е включена."})
    end

    if message = LoginAttemptTracker.lock_message("#{user.email}:2fa")
      response.status_code = 429
      return json({success: false, message: message})
    end

    code = json_field(json_body, "code")
    if ::TwoFactor.confirm(user, code)
      AuditLogger.log(context, "two_factor_enabled", user.email)
      json({
        success:                   true,
        message:                   "Двуфакторната защита е включена.",
        recovery_codes_remaining:  ::RecoveryCodes.remaining(user.otp_recovery_codes),
      })
    else
      LoginAttemptTracker.record_failure("#{user.email}:2fa".downcase)
      AuditLogger.log(context, "two_factor_confirm_failed", user.email)
      response.status_code = 422
      json({success: false, message: "Невалиден код. Опитайте с текущия код от приложението."})
    end
  end
end

# POST /api/auth/two_factor/disable
class Api::Auth::TwoFactor::Disable < ApiAction
  include Api::Auth::JsonBody

  post "/api/auth/two_factor/disable" do
    user = current_user?
    unless user
      response.status_code = 401
      return json({success: false, message: "Не сте влезли в системата"})
    end

    unless user.otp_enabled
      response.status_code = 422
      return json({success: false, message: "Двуфакторната защита не е включена."})
    end

    if message = LoginAttemptTracker.lock_message(user.email) || LoginAttemptTracker.lock_message("#{user.email}:2fa")
      response.status_code = 429
      return json({success: false, message: message})
    end

    body = json_body
    password = json_field(body, "password")
    code = json_field(body, "code")

    unless Authentic.correct_password?(user, password)
      LoginAttemptTracker.record_failure(user.email.downcase)
      response.status_code = 422
      return json({success: false, message: "Паролата е грешна"})
    end

    unless ::TwoFactor.authenticate(user, code)
      LoginAttemptTracker.record_failure("#{user.email}:2fa".downcase)
      response.status_code = 422
      return json({success: false, message: "Невалиден код"})
    end

    ::TwoFactor.disable(user)
    AuditLogger.log(context, "two_factor_disabled", user.email)
    json({success: true, message: "Двуфакторната защита е изключена."})
  end
end

# POST /api/auth/two_factor/recovery_codes
class Api::Auth::TwoFactor::RecoveryCodes < ApiAction
  include Api::Auth::JsonBody

  post "/api/auth/two_factor/recovery_codes" do
    user = current_user?
    unless user
      response.status_code = 401
      return json({success: false, message: "Не сте влезли в системата"})
    end

    unless user.otp_enabled
      response.status_code = 422
      return json({success: false, message: "Първо включете двуфакторната защита."})
    end

    if message = LoginAttemptTracker.lock_message(user.email) || LoginAttemptTracker.lock_message("#{user.email}:2fa")
      response.status_code = 429
      return json({success: false, message: message})
    end

    body = json_body
    password = json_field(body, "password")
    code = json_field(body, "code")

    unless Authentic.correct_password?(user, password)
      LoginAttemptTracker.record_failure(user.email.downcase)
      response.status_code = 422
      return json({success: false, message: "Паролата е грешна"})
    end

    unless ::TwoFactor.authenticate(user, code)
      LoginAttemptTracker.record_failure("#{user.email}:2fa".downcase)
      response.status_code = 422
      return json({success: false, message: "Невалиден код"})
    end

    codes = ::TwoFactor.replace_recovery_codes(user)
    AuditLogger.log(context, "two_factor_recovery_regenerated", user.email)
    json({
      success:         true,
      recovery_codes:  codes,
      message:         "Новите кодове заменят предишните. Изтеглете ги сега.",
    })
  end
end
