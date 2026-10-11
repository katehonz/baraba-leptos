# Enrollment and checks for a user's authenticator and recovery codes.
class TwoFactor
  struct Enrollment
    getter secret : String
    getter uri : String
    getter qr_image : String
    getter codes : Array(String)

    def initialize(@secret : String, @uri : String, @qr_image : String, @codes : Array(String))
    end
  end

  def self.pending?(user : User) : Bool
    !user.otp_enabled && !user.otp_secret.nil? && !user.otp_secret.not_nil!.blank?
  end

  def self.begin_enrollment(user : User) : Enrollment
    secret = Totp.generate_secret
    codes = RecoveryCodes.generate
    SaveUser.update!(user,
      otp_secret: secret,
      otp_enabled: false,
      otp_last_timestep: nil,
      otp_recovery_codes: RecoveryCodes.hashed_json(codes),
    )
    uri = Totp.provisioning_uri(secret, user.email)
    Enrollment.new(secret, uri, Totp.qr_data_url(uri), codes)
  end

  def self.confirm(user : User, code : String) : Bool
    return false if user.otp_enabled
    secret = user.otp_secret
    return false if secret.nil? || secret.blank?
    step = Totp.matching_step(secret, code, last_step: user.otp_last_timestep)
    return false unless step

    SaveUser.update!(user, otp_enabled: true, otp_last_timestep: step)
    true
  end

  # Accepts a current authenticator code or a unused recovery code.
  def self.authenticate(user : User, code : String) : Bool
    return false unless user.otp_enabled
    secret = user.otp_secret
    return false if secret.nil? || secret.blank?

    compact = code.strip
    if compact.size == Totp::DIGITS && compact.each_char.all?(&.ascii_number?)
      if step = Totp.matching_step(secret, compact, last_step: user.otp_last_timestep)
        SaveUser.update!(user, otp_last_timestep: step)
        return true
      end
      return false
    end

    RecoveryCodes.consume!(user, compact)
  end

  def self.replace_recovery_codes(user : User) : Array(String)
    codes = RecoveryCodes.generate
    SaveUser.update!(user, otp_recovery_codes: RecoveryCodes.hashed_json(codes))
    codes
  end

  def self.disable(user : User) : Nil
    SaveUser.update!(user,
      otp_secret: nil,
      otp_enabled: false,
      otp_last_timestep: nil,
      otp_recovery_codes: nil,
    )
  end
end
