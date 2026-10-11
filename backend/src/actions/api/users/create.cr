class Api::Users::Create < ApiAction
  include Api::Auth::RequireAuthToken

  post "/api/users" do
    current = current_user?.not_nil!

    unless current.is_super_admin
      response.status_code = 403
      return json({success: false, message: "Нямате достъп"})
    end

    email = params.get?(:email).to_s
    password = params.get?(:password).to_s
    first_name = params.get?(:first_name)
    last_name = params.get?(:last_name)

    if email.empty? || password.empty?
      json({success: false, message: "Email и парола са задължителни"})
    else
      # Check if user exists
      if UserQuery.new.email(email).first?
        json({success: false, message: "Потребител с този email вече съществува"})
      else
        encrypted = Authentic.generate_encrypted_password(password)

        user = SaveUser.create!(
          email: email,
          encrypted_password: encrypted,
          first_name: first_name,
          last_name: last_name,
          is_active: true
        )

        json({
          success: true,
          data:    {
            id:         user.id,
            email:      user.email,
            first_name: user.first_name,
            last_name:  user.last_name,
            is_active:  user.is_active,
          },
        })
      end
    end
  end
end
