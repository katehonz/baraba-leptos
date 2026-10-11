class ImproveRegistrationFlow::V20260201120000 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(User) do
      # Store company data as JSON until email is verified
      add pending_company_data : String?

      # Track when verification email was sent (for rate limiting)
      add verification_email_sent_at : Time?
    end
  end

  def rollback
    alter table_for(User) do
      remove :pending_company_data
      remove :verification_email_sent_at
    end
  end
end
