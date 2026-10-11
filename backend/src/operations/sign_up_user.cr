class SignUpUser < User::SaveOperation
  param_key :user
  permit_columns email, first_name, last_name, email_verification_token

  attribute password : String
  attribute password_confirmation : String

  before_save do
    validate_required password
    validate_size_of password, min: 8

    if password.value != password_confirmation.value
      password_confirmation.add_error("must match password")
    end

    Authentic.copy_and_encrypt(password, to: encrypted_password) if password.value

    # Generate verification token if not set
    if email_verification_token.value.nil?
      email_verification_token.value = Random::Secure.hex(32)
    end

    # IMPORTANT: Explicitly set is_super_admin to false to prevent any possibility
    # of mass assignment or default value issues
    is_super_admin.value = false
  end
end
