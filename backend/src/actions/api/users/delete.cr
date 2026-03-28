class Api::Users::Delete < ApiAction
  include Api::Auth::RequireAuthToken

  delete "/api/users/:user_id" do
    current = current_user?.not_nil!

    # Only super admins can delete users
    unless current.is_super_admin
      response.status_code = 403
      return json({success: false, message: "Само супер администратори могат да изтриват потребители"})
    end

    target_user_id = user_id.to_i64

    # Prevent self-deletion
    if target_user_id == current.id
      response.status_code = 400
      return json({success: false, message: "Не можете да изтриете себе си"})
    end

    user = UserQuery.new.id(target_user_id).first?

    unless user
      response.status_code = 404
      return json({success: false, message: "Потребителят не е намерен"})
    end

    # Prevent deleting other super admins
    if user.is_super_admin
      response.status_code = 403
      return json({success: false, message: "Не можете да изтриете друг супер администратор"})
    end

    # Delete user company roles first
    UserCompanyRoleQuery.new.user_id(target_user_id).each do |ucr|
      ucr.delete
    end

    # Delete the user
    user.delete

    AuditLogger.log(context, "delete_user", user.email)

    json({
      success: true,
      message: "Потребителят е изтрит",
    })
  end
end
