# POST /api/auth/signup - Register user without company
# Also handles POST /api/auth/register for backwards compatibility
class Api::Auth::SignUp::Create < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/auth/signup" do
    register_user
  end

  post "/api/auth/register" do
    register_user
  end

  private def register_user
    # Check if registration is enabled
    unless SystemSetting.registration_enabled?
      response.status_code = 403
      return json({success: false, message: "Регистрацията е временно спряна."})
    end

    SignUpUser.create(params) do |operation, user|
      if user
        # Do NOT return auth token — require email verification first
        json({
          success:               true,
          requires_verification: true,
          user:                  {
            id:         user.id,
            email:      user.email,
            first_name: user.first_name,
            last_name:  user.last_name,
          },
          message: "Регистрацията е успешна! Моля, проверете имейла си за потвърждение.",
        })
      else
        response.status_code = 422
        json({
          success: false,
          message: "Грешка при регистрация",
          errors:  operation.errors.map { |key, errors| {field: key, messages: errors} },
        })
      end
    end
  end
end
