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

    user_id = payload["user_id"]?.try(&.as_i64?)
    return nil unless user_id

    UserQuery.new.id(user_id).first?
  end

  private def self.secret_key : String
    Lucky::Server.settings.secret_key_base
  end
end
