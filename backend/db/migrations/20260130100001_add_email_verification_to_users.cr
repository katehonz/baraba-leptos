class AddEmailVerificationToUsers::V20260130100001 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(User) do
      add email_verification_token : String?
      add email_verified_at : Time?
    end
  end

  def rollback
    alter table_for(User) do
      remove :email_verification_token
      remove :email_verified_at
    end
  end
end
