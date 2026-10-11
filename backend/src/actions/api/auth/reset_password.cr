# POST /api/auth/reset_password - Reset password with token
class Api::Auth::ResetPassword < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/auth/reset_password" do
    token = params.get?("token") || ""
    new_password = params.get?("password") || ""
    confirm_password = params.get?("password_confirmation") || new_password

    # Validate inputs
    if token.empty?
      response.status_code = 422
      return json({success: false, message: "Невалиден или липсващ токен"})
    end

    if new_password.size < 8
      response.status_code = 422
      return json({success: false, message: "Паролата трябва да е поне 8 символа"})
    end

    if new_password != confirm_password
      response.status_code = 422
      return json({success: false, message: "Паролите не съвпадат"})
    end

    # Validate token and get user
    user = PasswordResetToken.user_from_token(token)

    unless user
      response.status_code = 422
      return json({success: false, message: "Невалиден или изтекъл токен. Моля заявете нов линк."})
    end

    # Update password
    encrypted = Authentic.generate_encrypted_password(new_password)
    SaveUser.update!(user, encrypted_password: encrypted)

    json({
      success: true,
      message: "Паролата е променена успешно! Можете да влезете с новата парола.",
    })
  end

  # GET /api/auth/reset_password/validate - Validate reset token
  get "/api/auth/reset_password/validate" do
    token = params.get?("token") || ""

    if token.empty?
      return json({valid: false, message: "Липсващ токен"})
    end

    user = PasswordResetToken.user_from_token(token)

    if user
      json({
        valid: true,
        email: user.email,
      })
    else
      json({
        valid:   false,
        message: "Невалиден или изтекъл токен",
      })
    end
  end
end
