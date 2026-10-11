class Api::Auth::SignIn::Create < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/auth/signin" do
    # Parse JSON body directly to avoid Lucky params issues
    body = request.body.try(&.gets_to_end) || "{}"
    json_data = JSON.parse(body)

    user_data = json_data["user"]?
    email = user_data.try(&.["email"]?).try(&.as_s) || ""
    password = user_data.try(&.["password"]?).try(&.as_s) || ""

    tracker_key = email.downcase

    # Account lockout after too many failed attempts
    if locked_until = LoginAttemptTracker.locked_until(tracker_key)
      AuditLogger.log(context, "login_locked", email)
      response.status_code = 429
      minutes = ((locked_until - Time.utc).total_minutes).ceil.to_i
      return json({
        success: false,
        message: "Твърде много неуспешни опити за вход. Акаунтът е заключен за още #{minutes} минути.",
      })
    end

    user = UserQuery.new.email(email).first?

    if user && !user.is_active
      AuditLogger.log(context, "login_inactive", email)
      response.status_code = 403
      return json({
        success: false,
        message: "Акаунтът е деактивиран. Свържете се с администратор.",
      })
    end

    if user && !password.blank? && Authentic.correct_password?(user, password)
      LoginAttemptTracker.record_success(tracker_key)

      # Check if email is verified
      unless user.email_verified?
        AuditLogger.log(context, "login_unverified", email)
        response.status_code = 403
        return json({
          success:               false,
          requires_verification: true,
          message:               "Моля, потвърдете вашия email преди да влезете.",
          email:                 user.email,
        })
      end

      if user.otp_enabled
        if user.otp_secret.nil? || user.otp_secret.not_nil!.blank?
          AuditLogger.log(context, "login_2fa_failed", email, "missing secret")
          response.status_code = 403
          return json({
            success: false,
            message: "Двуфакторната защита е включена, но ключът липсва. Свържете се с администратор.",
          })
        end

        if message = LoginAttemptTracker.lock_message("#{email}:2fa")
          AuditLogger.log(context, "login_locked", email)
          response.status_code = 429
          return json({success: false, message: message})
        end

        AuditLogger.log(context, "login_2fa_required", email)
        return json({
          success:         true,
          requires_2fa:    true,
          challenge_token: UserToken.generate_2fa_challenge(user),
          message:         "Въведете кода от приложението за удостоверяване или код за възстановяване.",
        })
      end

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
      LoginAttemptTracker.record_failure(tracker_key)
      AuditLogger.log(context, "login_failed", email)
      response.status_code = 401
      json({
        success: false,
        errors:  [{field: "email", messages: ["or password is incorrect"]}],
      })
    end
  end
end
