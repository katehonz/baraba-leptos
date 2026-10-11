class Api::Users::Update < ApiAction
  include Api::Auth::RequireAuthToken

  put "/api/users/:user_id" do
    current = current_user?.not_nil!

    unless current.is_super_admin
      response.status_code = 403
      return json({success: false, message: "Нямате достъп"})
    end

    user = UserQuery.new.id(user_id).first?

    if user.nil?
      json({success: false, message: "Потребителят не е намерен"})
    else
      SaveUser.update(user,
        first_name: params.get?(:first_name) || user.first_name,
        last_name: params.get?(:last_name) || user.last_name,
        email: params.get?(:email) || user.email
      ) do |op, updated_user|
        if op.saved?
          json({
            success: true,
            data:    {
              id:         updated_user.id,
              email:      updated_user.email,
              first_name: updated_user.first_name,
              last_name:  updated_user.last_name,
              is_active:  updated_user.is_active,
            },
          })
        else
          json({success: false, message: "Грешка при обновяване"})
        end
      end
    end
  end
end
