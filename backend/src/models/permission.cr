# Permission constants and utilities
# Migrated from old Java Permission enum
module Permission
  # Admin permissions
  ADMIN_READ    = "admin:read"
  ADMIN_CREATE  = "admin:create"
  ADMIN_UPDATE  = "admin:update"
  ADMIN_DELETE  = "admin:delete"

  # Accounting permissions
  ACCOUNTING_READ  = "accounting:read"
  ACCOUNTING_WRITE = "accounting:write"
  ACCOUNTING_POST  = "accounting:post" # Ability to finalize/post entries

  # Document permissions
  DOCUMENT_READ    = "document:read"
  DOCUMENT_UPLOAD  = "document:upload"
  DOCUMENT_PROCESS = "document:process"

  # Report permissions
  REPORT_READ   = "report:read"
  REPORT_EXPORT = "report:export"

  # Invoice permissions
  INVOICE_READ   = "invoice:read"
  INVOICE_CREATE = "invoice:create"
  INVOICE_UPDATE = "invoice:update"
  INVOICE_DELETE = "invoice:delete"
  INVOICE_SEND   = "invoice:send"

  # Payment permissions
  PAYMENT_READ   = "payment:read"
  PAYMENT_CREATE = "payment:create"
  PAYMENT_UPDATE = "payment:update"
  PAYMENT_DELETE = "payment:delete"

  # VAT permissions
  VAT_READ   = "vat:read"
  VAT_CREATE = "vat:create"
  VAT_SUBMIT = "vat:submit"

  # Settings permissions
  SETTINGS_READ   = "settings:read"
  SETTINGS_UPDATE = "settings:update"

  # User management permissions
  USER_READ   = "user:read"
  USER_CREATE = "user:create"
  USER_UPDATE = "user:update"
  USER_DELETE = "user:delete"

  # Company permissions
  COMPANY_READ   = "company:read"
  COMPANY_CREATE = "company:create"
  COMPANY_UPDATE = "company:update"
  COMPANY_DELETE = "company:delete"

  # Role permissions
  ROLE_READ   = "role:read"
  ROLE_CREATE = "role:create"
  ROLE_UPDATE = "role:update"
  ROLE_DELETE = "role:delete"

  # Wildcard
  ALL = "*"

  # All permissions list
  ALL_PERMISSIONS = [
    ADMIN_READ, ADMIN_CREATE, ADMIN_UPDATE, ADMIN_DELETE,
    ACCOUNTING_READ, ACCOUNTING_WRITE, ACCOUNTING_POST,
    DOCUMENT_READ, DOCUMENT_UPLOAD, DOCUMENT_PROCESS,
    REPORT_READ, REPORT_EXPORT,
    INVOICE_READ, INVOICE_CREATE, INVOICE_UPDATE, INVOICE_DELETE, INVOICE_SEND,
    PAYMENT_READ, PAYMENT_CREATE, PAYMENT_UPDATE, PAYMENT_DELETE,
    VAT_READ, VAT_CREATE, VAT_SUBMIT,
    SETTINGS_READ, SETTINGS_UPDATE,
    USER_READ, USER_CREATE, USER_UPDATE, USER_DELETE,
    COMPANY_READ, COMPANY_CREATE, COMPANY_UPDATE, COMPANY_DELETE,
    ROLE_READ, ROLE_CREATE, ROLE_UPDATE, ROLE_DELETE,
  ]

  # Permission groups for easy assignment
  ADMIN_ALL = [ADMIN_READ, ADMIN_CREATE, ADMIN_UPDATE, ADMIN_DELETE]
  ACCOUNTING_ALL = [ACCOUNTING_READ, ACCOUNTING_WRITE, ACCOUNTING_POST]
  DOCUMENT_ALL = [DOCUMENT_READ, DOCUMENT_UPLOAD, DOCUMENT_PROCESS]
  INVOICE_ALL = [INVOICE_READ, INVOICE_CREATE, INVOICE_UPDATE, INVOICE_DELETE, INVOICE_SEND]
  PAYMENT_ALL = [PAYMENT_READ, PAYMENT_CREATE, PAYMENT_UPDATE, PAYMENT_DELETE]
  VAT_ALL = [VAT_READ, VAT_CREATE, VAT_SUBMIT]
  REPORT_ALL = [REPORT_READ, REPORT_EXPORT]
  USER_ALL = [USER_READ, USER_CREATE, USER_UPDATE, USER_DELETE]
  COMPANY_ALL = [COMPANY_READ, COMPANY_CREATE, COMPANY_UPDATE, COMPANY_DELETE]
  ROLE_ALL = [ROLE_READ, ROLE_CREATE, ROLE_UPDATE, ROLE_DELETE]

  # Read-only permissions
  READ_ONLY = [
    ADMIN_READ, ACCOUNTING_READ, DOCUMENT_READ, REPORT_READ,
    INVOICE_READ, PAYMENT_READ, VAT_READ, SETTINGS_READ,
    USER_READ, COMPANY_READ, ROLE_READ,
  ]

  # Check if a permission string matches a required permission
  # Supports wildcards like "*" (all) and "accounting:*" (all accounting)
  def self.matches?(user_permission : String, required : String) : Bool
    return true if user_permission == ALL
    return true if user_permission == required

    # Check for category wildcard like "accounting:*"
    if user_permission.ends_with?(":*")
      category = user_permission.rchop(":*")
      return required.starts_with?("#{category}:")
    end

    false
  end

  # Check if user has any of the required permissions
  def self.has_permission?(user_permissions : Array(String), required : String) : Bool
    user_permissions.any? { |p| matches?(p, required) }
  end

  # Check if user has all of the required permissions
  def self.has_all_permissions?(user_permissions : Array(String), required : Array(String)) : Bool
    required.all? { |r| has_permission?(user_permissions, r) }
  end

  # Check if user has any of the required permissions
  def self.has_any_permission?(user_permissions : Array(String), required : Array(String)) : Bool
    required.any? { |r| has_permission?(user_permissions, r) }
  end
end
