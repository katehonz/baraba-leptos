class Api::Users::ToggleActive < ApiAction
  include Api::Auth::RequireAuthToken

  post "/api/users/:user_id/toggle_active" do
    current = current_user?.not_nil!

    unless current.is_super_admin
      response.status_code = 403
      return json({success: false, message: "Само супер администратори могат да променят статуса на потребители"})
    end

    user = UserQuery.new.id(user_id).first?

    if user.nil?
      json({success: false, message: "Потребителят не е намерен"})
    else
      new_status = !user.is_active
      SaveUser.update!(user, is_active: new_status)

      json({
        success:   true,
        is_active: new_status,
        message:   new_status ? "Потребителят е активиран" : "Потребителят е деактивиран",
      })
    end
  end
end
