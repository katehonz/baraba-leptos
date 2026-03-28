class AuditLog < BaseModel
  table do
    column event : String
    column email : String
    column ip_address : String
    column user_agent : String?
    column details : String?
  end
end
