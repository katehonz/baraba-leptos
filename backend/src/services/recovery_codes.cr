require "json"
require "openssl"
require "crypto/subtle"

# One-time recovery codes. Only HMAC hashes are stored.
class RecoveryCodes
  COUNT    = 8
  LENGTH   = 8
  ALPHABET = "abcdefghjkmnpqrstuvwxyz23456789"

  def self.generate(count : Int32 = COUNT) : Array(String)
    Array.new(count) { generate_one }
  end

  def self.hashed_json(codes : Array(String)) : String
    codes.map { |code| hash(code) }.to_json
  end

  def self.remaining(raw : String?) : Int32
    hashes(raw).size
  end

  # Removes one matching code. Returns false when none match.
  def self.consume!(user : User, code : String) : Bool
    normalized = normalize(code)
    return false unless normalized.size == LENGTH

    target = hash(normalized)
    stored = hashes(user.otp_recovery_codes)
    matched_at = nil
    stored.each_with_index do |item, index|
      if Crypto::Subtle.constant_time_compare(item, target)
        matched_at = index
      end
    end
    return false unless matched_at

    stored.delete_at(matched_at)
    SaveUser.update!(user, otp_recovery_codes: stored.to_json)
    true
  end

  def self.hash(code : String) : String
    OpenSSL::HMAC.hexdigest(OpenSSL::Algorithm::SHA256, pepper, normalize(code))
  end

  def self.normalize(code : String) : String
    code.downcase.gsub(/[^a-z0-9]/, "")
  end

  def self.hashes(raw : String?) : Array(String)
    return [] of String if raw.nil? || raw.blank?
    JSON.parse(raw).as_a.map(&.as_s)
  rescue
    [] of String
  end

  def self.pepper : String
    Lucky::Server.settings.secret_key_base
  end

  private def self.generate_one : String
    chars = String.build do |io|
      LENGTH.times do
        io << ALPHABET[Random::Secure.rand(ALPHABET.size)]
      end
    end
    "#{chars[0..3]}-#{chars[4..7]}"
  end
end
