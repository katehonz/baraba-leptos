class SystemSetting < BaseModel
  skip_schema_enforcer
  table do
    column key : String
    column value : String = "{}"
    column description : String?
  end

  # Helper to get setting by key
  def self.get(key : String) : SystemSetting?
    SystemSettingQuery.new.key(key).first?
  end

  # Helper to get value as JSON
  def parsed_value : JSON::Any
    return JSON::Any.new({} of String => JSON::Any) if self.value.empty?
    begin
      JSON.parse(self.value)
    rescue
      JSON::Any.new({} of String => JSON::Any)
    end
  end

  # SMTP Settings helpers
  def self.smtp_settings : JSON::Any
    setting = get("smtp")
    setting ? setting.parsed_value : JSON::Any.new({} of String => JSON::Any)
  end

  def self.smtp_enabled? : Bool
    smtp_settings.dig?("enabled").try(&.as_bool?) || false
  end

  def self.smtp_host : String?
    smtp_settings.dig?("host").try(&.as_s?)
  end

  def self.smtp_port : Int32
    smtp_settings.dig?("port").try(&.as_i?) || 587
  end

  def self.smtp_username : String?
    smtp_settings.dig?("username").try(&.as_s?)
  end

  def self.smtp_password : String?
    smtp_settings.dig?("password").try(&.as_s?)
  end

  def self.smtp_from_email : String?
    smtp_settings.dig?("from_email").try(&.as_s?)
  end

  def self.smtp_from_name : String?
    smtp_settings.dig?("from_name").try(&.as_s?)
  end

  def self.smtp_use_tls? : Bool
    smtp_settings.dig?("use_tls").try(&.as_bool?) || true
  end

  # Application Settings helpers
  def self.app_settings : JSON::Any
    setting = get("app")
    setting ? setting.parsed_value : JSON::Any.new({} of String => JSON::Any)
  end

  def self.app_name : String
    app_settings.dig?("name").try(&.as_s?) || "Baraba"
  end

  def self.app_url : String?
    app_settings.dig?("url").try(&.as_s?)
  end

  def self.default_language : String
    app_settings.dig?("default_language").try(&.as_s?) || "bg"
  end

  def self.registration_enabled? : Bool
    app_settings.dig?("registration_enabled").try(&.as_bool?) || false
  end

  # SEO settings helpers
  def self.site_description : String?
    app_settings.dig?("site_description").try(&.as_s?)
  end

  def self.meta_keywords : String?
    app_settings.dig?("meta_keywords").try(&.as_s?)
  end

  def self.og_image_url : String?
    app_settings.dig?("og_image_url").try(&.as_s?)
  end

  def self.favicon_url : String?
    app_settings.dig?("favicon_url").try(&.as_s?)
  end

  def self.footer_text : String?
    app_settings.dig?("footer_text").try(&.as_s?)
  end

  # Update or create setting
  def self.set(key : String, value : Hash | NamedTuple, description : String? = nil)
    existing = get(key)
    if existing
      SaveSystemSetting.update!(existing, value: value.to_json, description: description || existing.description)
    else
      SaveSystemSetting.create!(key: key, value: value.to_json, description: description)
    end
  end
end
