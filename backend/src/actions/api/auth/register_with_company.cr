# POST /api/auth/register-with-company - Register user with a new company
# Company is created AFTER email verification
class Api::Auth::RegisterWithCompany < ApiAction
  include Api::Auth::SkipRequireAuthToken

  Log = ::Log.for("registration")

  post "/api/auth/register_with_company" do
    # Check if registration is enabled
    unless SystemSetting.registration_enabled?
      response.status_code = 403
      return json({success: false, message: "Регистрацията е временно спряна."})
    end

    # Parse JSON body
    json_params = params.from_json

    # Get user params
    user_email = json_params["user:email"]?.try(&.as_s?) || ""
    user_password = json_params["user:password"]?.try(&.as_s?) || ""
    user_password_confirmation = json_params["user:password_confirmation"]?.try(&.as_s?) || user_password
    user_first_name = json_params["user:first_name"]?.try(&.as_s?)
    user_last_name = json_params["user:last_name"]?.try(&.as_s?)

    # Get company params
    company_name = json_params["company:name"]?.try(&.as_s?) || ""
    company_eik = json_params["company:eik"]?.try(&.as_s?) || ""
    company_vat = json_params["company:vat_number"]?.try(&.as_s?)
    company_address = json_params["company:address"]?.try(&.as_s?)
    company_city = json_params["company:city"]?.try(&.as_s?)

    # Validate required fields
    if user_email.empty?
      response.status_code = 422
      return json({success: false, message: "Email е задължителен"})
    end

    if company_name.empty?
      response.status_code = 422
      return json({success: false, message: "Име на фирмата е задължително"})
    end

    # Check if email already exists
    existing_user = UserQuery.new.email(user_email).first?
    if existing_user
      AuditLogger.log(context, "register_duplicate", user_email)
      response.status_code = 422
      return json({success: false, message: "Потребител с този email вече съществува"})
    end

    begin
      # Store company data as JSON for later creation (after email verification)
      pending_company = {
        name:       company_name,
        eik:        company_eik,
        vat_number: company_vat,
        address:    company_address,
        city:       company_city,
      }.to_json

      # Create the user with pending company data
      user = SignUpUser.create!(
        email: user_email,
        password: user_password,
        password_confirmation: user_password_confirmation,
        first_name: user_first_name,
        last_name: user_last_name
      )

      # Update with pending company data and verification timestamp
      SaveUser.update!(user,
        pending_company_data: pending_company,
        verification_email_sent_at: Time.utc
      )

      # Reload user to get updated fields
      reloaded_user = UserQuery.new.id(user.id).first?
      user = reloaded_user.not_nil!

      # Send verification email
      app_url = SystemSetting.app_url
      if app_url.nil? || app_url.empty?
        response.status_code = 500
        return json({
          success: false,
          message: "Системата не е конфигурирана правилно. Моля, свържете се с администратора.",
        })
      end

      verify_url = "#{app_url}/verify-email?token=#{user.email_verification_token}"

      # Check SMTP configuration before attempting to send
      smtp_enabled = SystemSetting.smtp_enabled?
      smtp_host = SystemSetting.smtp_host

      Log.info { "Registration email attempt: user=#{user.email}, smtp_enabled=#{smtp_enabled}, smtp_host=#{smtp_host}" }

      email_sent = false
      email_error = nil

      if smtp_enabled
        email_sent = EmailService.send_verification_email(user, verify_url)
        Log.info { "Email send result: #{email_sent}" }
      else
        email_error = "SMTP не е активиран в системните настройки"
        Log.warn { "SMTP is not enabled, skipping verification email" }
      end

      AuditLogger.log(context, "register", user_email, {company: company_name, eik: company_eik}.to_json)

      json({
        success:               true,
        requires_verification: true,
        email_sent:            email_sent,
        user:                  {
          id:         user.id,
          email:      user.email,
          first_name: user.first_name,
          last_name:  user.last_name,
        },
        message: email_sent ?
          "Регистрацията е успешна! Изпратихме email за потвърждение на #{user.email}. Моля, проверете пощата си." :
          "Регистрацията е успешна, но не успяхме да изпратим email. Моля, свържете се с администратора.",
      })
    rescue ex : Avram::InvalidOperationError
      response.status_code = 422
      json({success: false, message: "Грешка при регистрация: #{ex.message}"})
    rescue ex
      response.status_code = 500
      json({success: false, message: "Грешка: #{ex.message}"})
    end
  end
end
