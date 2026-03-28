# POST /api/auth/forgot_password - Request password reset email
class Api::Auth::ForgotPassword < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/auth/forgot_password" do
    email = params.get?("email") || ""

    if email.empty?
      response.status_code = 422
      return json({success: false, message: "Email е задължителен"})
    end

    # Find user by email
    user = UserQuery.new.email(email).first?

    if user
      # Check if app_url is configured
      app_url = SystemSetting.app_url
      if app_url.nil? || app_url.empty?
        Log.warn { "app_url not configured, cannot send password reset email" }
      else
        # Generate reset token
        token = PasswordResetToken.generate(user)

        # Build reset URL
        reset_url = "#{app_url}/reset-password?token=#{token}"

        # Send email
        if EmailService.send_password_reset(user, reset_url)
          # Email sent successfully
        else
          # Log error but don't reveal to user
          Log.warn { "Failed to send password reset email to #{email}" }
        end
      end
    end

    AuditLogger.log(context, "forgot_password", email)

    # Always return success to prevent email enumeration
    json({
      success: true,
      message: "Ако съществува акаунт с този email, ще получите линк за възстановяване.",
    })
  end
end
