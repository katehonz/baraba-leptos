class SaveUser < User::SaveOperation
  permit_columns email, encrypted_password, first_name, last_name, is_active,
                 email_verification_token, email_verified_at,
                 pending_company_data, verification_email_sent_at
end
