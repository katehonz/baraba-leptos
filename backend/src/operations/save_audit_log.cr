class SaveAuditLog < AuditLog::SaveOperation
  permit_columns event, email, ip_address, user_agent, details
end
