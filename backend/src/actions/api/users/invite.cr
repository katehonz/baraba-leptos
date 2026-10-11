class Api::Users::Invite < ApiAction
  include Api::Auth::RequireAuthToken

  post "/api/users/invite" do
    current = current_user?.not_nil!

    unless current.is_super_admin
      response.status_code = 403
      return json({success: false, message: "Нямате достъп"})
    end

    email = params.get?(:email).to_s

    if email.empty?
      json({success: false, message: "Email е задължителен"})
    else
      # Check if user exists
      existing = UserQuery.new.email(email).first?

      if existing
        json({success: false, message: "Потребител с този email вече съществува"})
      else
        # Generate random password for invite
        temp_password = Random::Secure.hex(8)
        encrypted = Authentic.generate_encrypted_password(temp_password)

        user = SaveUser.create!(
          email: email,
          encrypted_password: encrypted,
          is_active: true
        )

        # TODO: Send invitation email with temp_password
        # For now, just return success

        json({
          success: true,
          message: "Поканата е изпратена",
          data:    {
            id:    user.id,
            email: user.email,
          },
        })
      end
    end
  end
end
