# PUT /api/auth/profile - Update current user profile
class Api::Auth::UpdateProfile < ApiAction
  put "/api/auth/profile" do
    if user = current_user?
      first_name = params.get?("first_name")
      last_name = params.get?("last_name")
      email = params.get?("email")

      SaveUser.update(user,
        first_name: first_name || user.first_name,
        last_name: last_name || user.last_name,
        email: email || user.email
      ) do |operation, updated_user|
        if updated_user
          json({
            success: true,
            message: "Профилът е обновен успешно",
            data:    {
              id:         updated_user.id,
              email:      updated_user.email,
              first_name: updated_user.first_name,
              last_name:  updated_user.last_name,
            },
          })
        else
          response.status_code = 422
          json({
            success: false,
            errors:  operation.errors.map { |key, errors| {field: key, messages: errors} },
          })
        end
      end
    else
      response.status_code = 401
      json({success: false, message: "Не сте влезли в системата"})
    end
  end
end
