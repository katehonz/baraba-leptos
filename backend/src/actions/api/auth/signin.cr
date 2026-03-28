class Api::Auth::SignIn::Create < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/auth/signin" do
    # Parse JSON body directly to avoid Lucky params issues
    body = request.body.try(&.gets_to_end) || "{}"
    json_data = JSON.parse(body)

    user_data = json_data["user"]?
    email = user_data.try(&.["email"]?).try(&.as_s) || ""
    password = user_data.try(&.["password"]?).try(&.as_s) || ""

    user = UserQuery.new.email(email).first?

    if user && !password.blank? && Authentic.correct_password?(user, password)
      # Check if email is verified
      unless user.email_verified?
        AuditLogger.log(context, "login_unverified", email)
        response.status_code = 403
        return json({
          success: false,
          requires_verification: true,
          message: "Моля, потвърдете вашия email преди да влезете.",
          email: user.email,
        })
      end

      AuditLogger.log(context, "login", email)

      token = UserToken.generate(user)
      json({
        success: true,
        token:   token,
        user:    {
          id:       user.id,
          email:    user.email,
          username: user.email.split('@').first,
        },
      })
    else
      AuditLogger.log(context, "login_failed", email)
      response.status_code = 401
      json({
        success: false,
        errors:  [{field: "email", messages: ["or password is incorrect"]}],
      })
    end
  end
end
