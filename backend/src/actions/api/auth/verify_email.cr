# GET /api/auth/verify-email?token=xxx - Verify user email
# Creates company if pending company data exists
class Api::Auth::VerifyEmail < ApiAction
  include Api::Auth::SkipRequireAuthToken

  get "/api/auth/verify_email" do
    token = params.get?(:token)

    unless token
      response.status_code = 400
      return json({success: false, message: "Token е задължителен"})
    end

    user = UserQuery.new.email_verification_token(token).first?

    unless user
      response.status_code = 404
      return json({success: false, message: "Невалиден или изтекъл token"})
    end

    if user.email_verified?
      return json({success: true, message: "Email вече е потвърден", already_verified: true})
    end

    begin
      # Mark email as verified
      SaveUser.update!(user, email_verified_at: Time.utc, email_verification_token: nil)

      # Create company if pending company data exists
      company_created = false
      company_data = nil

      if user.has_pending_company?
        pending = user.parsed_pending_company_data.not_nil!

        # Create the company with user as owner
        company = SaveCompany.create!(
          name: pending["name"]?.try(&.as_s?) || "Моята фирма",
          eik: pending["eik"]?.try(&.as_s?) || "",
          vat_number: pending["vat_number"]?.try(&.as_s?),
          address: pending["address"]?.try(&.as_s?),
          city: pending["city"]?.try(&.as_s?),
          owner_id: user.id
        )

        # Initialize company accounts from SAF-T
        init_result = CompanyAccountsInitializer.initialize_accounts(company)

        # Get or create admin role (with specific permissions, NOT wildcard "*")
        admin_role = RoleQuery.new.name("admin").first?
        unless admin_role
          admin_permissions = (Permission::ADMIN_ALL + Permission::USER_ALL + Permission::COMPANY_ALL + Permission::ROLE_ALL + [Permission::SETTINGS_READ, Permission::SETTINGS_UPDATE] + Permission::REPORT_ALL + Permission::ACCOUNTING_ALL + Permission::INVOICE_ALL + Permission::PAYMENT_ALL + Permission::VAT_ALL + Permission::DOCUMENT_ALL).to_json
          admin_role = SaveRole.create!(
            name: "admin",
            description: "Администратор на фирма",
            permissions: admin_permissions
          )
        end

        # Assign user as admin of the company
        SaveUserCompanyRole.create!(
          user_id: user.id,
          company_id: company.id,
          role_id: admin_role.id
        )

        # Clear pending company data
        SaveUser.update!(user, pending_company_data: nil)

        company_created = true
        company_data = {
          id:   company.id,
          name: company.name,
          eik:  company.eik,
        }
      end

      AuditLogger.log(context, "verify_email", user.email, company_created ? "company_created" : nil)

      # Generate auth token for auto-login after verification
      auth_token = UserToken.generate(user)

      json({
        success:         true,
        message:         company_created ?
          "Email потвърден успешно! Фирмата е създадена." :
          "Email потвърден успешно!",
        token:           auth_token,
        company_created: company_created,
        company:         company_data,
        user:            {
          id:         user.id,
          email:      user.email,
          first_name: user.first_name,
          last_name:  user.last_name,
        },
      })
    rescue ex
      response.status_code = 500
      json({success: false, message: "Грешка при активиране: #{ex.message}"})
    end
  end
end
