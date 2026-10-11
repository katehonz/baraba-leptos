class CreateAuditLogs::V20260215000001 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(AuditLog) do
      primary_key id : Int64
      add_timestamps
      add event : String          # register, login, login_failed, verify_email, forgot_password
      add email : String
      add ip_address : String
      add user_agent : String?
      add details : String?       # JSON extra info
    end

    execute "CREATE INDEX idx_audit_logs_event ON audit_logs (event)"
    execute "CREATE INDEX idx_audit_logs_email ON audit_logs (email)"
    execute "CREATE INDEX idx_audit_logs_ip ON audit_logs (ip_address)"
    execute "CREATE INDEX idx_audit_logs_created ON audit_logs (created_at DESC)"
  end

  def rollback
    drop table_for(AuditLog)
  end
end
