require "jwt"

# Service for generating and validating password reset tokens
# Tokens are JWT with short expiration (1 hour)
class PasswordResetToken
  ALGORITHM   = JWT::Algorithm::HS256
  EXPIRATION  = 1.hour  # Token valid for 1 hour

  # Generate a password reset token for user
  def self.generate(user : User) : String
    payload = {
      "user_id"  => user.id,
      "email"    => user.email,
      "purpose"  => "password_reset",
      "exp"      => (Time.utc + EXPIRATION).to_unix,
      "iat"      => Time.utc.to_unix,
    }

    JWT.encode(payload, secret_key, ALGORITHM)
  end

  # Decode and validate token, returns user_id if valid
  def self.decode(token : String) : Int64?
    payload, _header = JWT.decode(token, secret_key, ALGORITHM)
    data = payload.as_h

    # Verify it's a password reset token
    return nil unless data["purpose"]?.try(&.as_s?) == "password_reset"

    data["user_id"]?.try(&.as_i64?)
  rescue JWT::ExpiredSignatureError
    nil
  rescue JWT::VerificationError
    nil
  rescue JWT::DecodeError
    nil
  end

  # Get user from password reset token
  def self.user_from_token(token : String) : User?
    user_id = decode(token)
    return nil unless user_id

    UserQuery.new.id(user_id).first?
  end

  private def self.secret_key : String
    # Use different key for password reset to prevent token reuse
    "#{Lucky::Server.settings.secret_key_base}_password_reset"
  end
end
