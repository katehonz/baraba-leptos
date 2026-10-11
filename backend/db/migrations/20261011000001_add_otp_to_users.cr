class AddOtpToUsers::V20261011000001 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(User) do
      add otp_secret : String?
      add otp_enabled : Bool, default: false
      add otp_last_timestep : Int64?
      add otp_recovery_codes : String?
    end
  end

  def rollback
    alter table_for(User) do
      remove :otp_recovery_codes
      remove :otp_last_timestep
      remove :otp_enabled
      remove :otp_secret
    end
  end
end
