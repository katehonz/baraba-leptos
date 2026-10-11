require "openssl"
require "crypto/subtle"
require "base64"

# RFC 6238 TOTP (SHA1, 6 digits, 30 seconds) and the otpauth QR image.
class Totp
  ISSUER     = "Baraba"
  DIGITS     = 6
  PERIOD     = 30
  WINDOW     = 1
  SECRET_BYTES = 20

  class Base32
    ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"

    def self.encode(data : Bytes) : String
      String.build do |io|
        buffer = 0_u64
        bits = 0
        data.each do |byte|
          buffer = (buffer << 8) | byte.to_u64
          bits += 8
          while bits >= 5
            bits -= 5
            io << ALPHABET[((buffer >> bits) & 31).to_i]
          end
        end
        if bits > 0
          io << ALPHABET[((buffer << (5 - bits)) & 31).to_i]
        end
      end
    end

    def self.decode(input : String) : Bytes
      cleaned = input.upcase.gsub(/[^A-Z2-7]/, "")
      bytes = IO::Memory.new
      buffer = 0_u64
      bits = 0
      cleaned.each_char do |char|
        index = ALPHABET.index(char)
        return Bytes.empty unless index
        buffer = (buffer << 5) | index.to_u64
        bits += 5
        if bits >= 8
          bits -= 8
          bytes.write_byte(((buffer >> bits) & 0xff).to_u8)
        end
      end
      bytes.to_slice
    end
  end

  def self.generate_secret : String
    Base32.encode(Random::Secure.random_bytes(SECRET_BYTES))
  end

  def self.provisioning_uri(secret : String, email : String) : String
    label = "#{URI.encode_www_form(ISSUER, space_to_plus: false)}:#{URI.encode_www_form(email, space_to_plus: false)}"
    issuer = URI.encode_www_form(ISSUER, space_to_plus: false)
    "otpauth://totp/#{label}?secret=#{secret}&issuer=#{issuer}&algorithm=SHA1&digits=#{DIGITS}&period=#{PERIOD}"
  end

  # SVG QR as a data URL so the profile can show it in an <img>.
  def self.qr_data_url(uri : String) : String
    svg = QRCode.new(uri, level: :m).as_svg(24, "000", "crispEdges", 6, true, "fff")
    "data:image/svg+xml;base64,#{Base64.strict_encode(svg)}"
  end

  def self.code_at(secret : String, at : Time, digits : Int32 = DIGITS) : String
    key = Base32.decode(secret)
    hotp(key, at.to_unix // PERIOD, digits)
  end

  # Returns the matched timestep, or nil when the code is wrong or already used.
  def self.matching_step(secret : String, code : String, at : Time = Time.utc, *, last_step : Int64? = nil, digits : Int32 = DIGITS, window : Int32 = WINDOW) : Int64?
    normalized = code.strip
    return nil unless normalized.size == digits && normalized.each_char.all?(&.ascii_number?)

    key = Base32.decode(secret)
    return nil if key.empty?

    timestep = at.to_unix // PERIOD
    offsets = [0]
    1.upto(window) { |i| offsets << -i << i }

    offsets.each do |offset|
      step = timestep + offset
      if used_step = last_step
        next if step <= used_step
      end
      expected = hotp(key, step, digits)
      return step if Crypto::Subtle.constant_time_compare(expected, normalized)
    end
    nil
  end

  def self.hotp(key : Bytes, counter : Int64, digits : Int32) : String
    hmac = OpenSSL::HMAC.digest(OpenSSL::Algorithm::SHA1, key, counter_bytes(counter))
    offset = hmac[-1].to_i & 0x0f
    binary = ((hmac[offset].to_i64 & 0x7f) << 24) |
             ((hmac[offset + 1].to_i64 & 0xff) << 16) |
             ((hmac[offset + 2].to_i64 & 0xff) << 8) |
             (hmac[offset + 3].to_i64 & 0xff)
    (binary % (10_i64 ** digits)).to_s.rjust(digits, '0')
  end

  def self.counter_bytes(counter : Int64) : Bytes
    bytes = Bytes.new(8)
    value = counter
    7.downto(0) do |i|
      bytes[i] = (value & 0xff).to_u8
      value >>= 8
    end
    bytes
  end
end
