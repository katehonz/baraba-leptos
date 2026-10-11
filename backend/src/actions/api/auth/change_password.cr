# PUT /api/auth/password - Change current user password
class Api::Auth::ChangePassword < ApiAction
  put "/api/auth/password" do
    if user = current_user?
      current_password = params.get?("current_password") || ""
      new_password = params.get?("new_password") || ""
      confirm_password = params.get?("confirm_password") || ""

      # Validate current password
      unless Authentic.correct_password?(user, current_password)
        response.status_code = 422
        return json({success: false, message: "Текущата парола е грешна"})
      end

      # Validate new password
      if new_password.size < 8
        response.status_code = 422
        return json({success: false, message: "Новата парола трябва да е поне 8 символа"})
      end

      if new_password != confirm_password
        response.status_code = 422
        return json({success: false, message: "Паролите не съвпадат"})
      end

      # Update password
      encrypted = Authentic.generate_encrypted_password(new_password)
      SaveUser.update!(user, encrypted_password: encrypted)

      json({success: true, message: "Паролата е променена успешно"})
    else
      response.status_code = 401
      json({success: false, message: "Не сте влезли в системата"})
    end
  end
end
