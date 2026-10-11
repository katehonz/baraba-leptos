require "jwt"

class UserToken
  ALGORITHM = JWT::Algorithm::HS256

  def self.generate(user : User) : String
    payload = {
      "user_id" => user.id,
      "email"   => user.email,
      "exp"     => (Time.utc + 7.days).to_unix,
    }

    JWT.encode(payload, secret_key, ALGORITHM)
  end

  # Short-lived token that only allows completing the second login step.
  def self.generate_2fa_challenge(user : User) : String
    payload = {
      "user_id" => user.id,
      "purpose" => "2fa",
      "exp"     => (Time.utc + 5.minutes).to_unix,
    }

    JWT.encode(payload, secret_key, ALGORITHM)
  end

  def self.decode(token : String) : Hash(String, JSON::Any)?
    payload, _header = JWT.decode(token, secret_key, ALGORITHM)
    payload.as_h
  rescue JWT::ExpiredSignatureError
    nil
  rescue JWT::VerificationError
    nil
  rescue JWT::DecodeError
    nil
  end

  def self.user_from_token(token : String) : User?
    payload = decode(token)
    return nil unless payload
    # A 2FA challenge must not open a session.
    return nil if payload["purpose"]?.try(&.as_s?) == "2fa"

    user_from_payload(payload)
  end

  def self.user_from_2fa_challenge(token : String) : User?
    payload = decode(token)
    return nil unless payload
    return nil unless payload["purpose"]?.try(&.as_s?) == "2fa"

    user_from_payload(payload)
  end

  private def self.user_from_payload(payload : Hash(String, JSON::Any)) : User?
    user_id = payload["user_id"]?.try(&.as_i64?)
    return nil unless user_id

    UserQuery.new.id(user_id).first?
  end

  private def self.secret_key : String
    Lucky::Server.settings.secret_key_base
  end
end
