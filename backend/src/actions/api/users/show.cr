class Api::Users::Show < ApiAction
  include Api::Auth::RequireAuthToken

  get "/api/users/:user_id" do
    current = current_user?.not_nil!

    unless current.is_super_admin
      response.status_code = 403
      return json({success: false, message: "Нямате достъп"})
    end

    user = UserQuery.new.id(user_id).first?

    if user
      json({
        success: true,
        data:    {
          id:         user.id,
          email:      user.email,
          first_name: user.first_name,
          last_name:  user.last_name,
          is_active:  user.is_active,
          created_at: user.created_at,
        },
      })
    else
      json({success: false, message: "Потребителят не е намерен"})
    end
  end
end
