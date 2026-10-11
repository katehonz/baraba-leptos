require "../spec_helper"

describe Totp do
  it "matches the RFC 6238 SHA1 vectors" do
    secret = Totp::Base32.encode("12345678901234567890".to_slice)
    Totp.code_at(secret, Time.unix(59), digits: 8).should eq("94287082")
    Totp.code_at(secret, Time.unix(1111111109), digits: 8).should eq("07081804")
    Totp.code_at(secret, Time.unix(1111111111), digits: 8).should eq("14050471")
    Totp.code_at(secret, Time.unix(1234567890), digits: 8).should eq("89005924")
    Totp.code_at(secret, Time.unix(2000000000), digits: 8).should eq("69279037")
    Totp.code_at(secret, Time.unix(20_000_000_000), digits: 8).should eq("65353130")
  end

  it "round-trips base32 without padding" do
    encoded = Totp::Base32.encode("foo".to_slice)
    encoded.should eq("MZXW6")
    String.new(Totp::Base32.decode(encoded)).should eq("foo")
  end

  it "renders a QR image for the provisioning URI" do
    secret = Totp.generate_secret
    uri = Totp.provisioning_uri(secret, "user@example.com")
    uri.should contain(secret)
    uri.should contain("issuer=Baraba")

    url = Totp.qr_data_url(uri)
    url.should start_with("data:image/svg+xml;base64,")
    svg = Base64.decode_string(url.split(",", 2)[1])
    svg.should contain("<svg")
    svg.should contain("<rect")
  end

  it "rejects a code from an already used timestep" do
    secret = Totp.generate_secret
    now = Time.utc
    code = Totp.code_at(secret, now)
    step = Totp.matching_step(secret, code, now).not_nil!
    Totp.matching_step(secret, code, now, last_step: step).should be_nil
  end
end

describe RecoveryCodes do
  it "accepts a code once, with or without the dash" do
    user = UserFactory.create
    codes = RecoveryCodes.generate
    SaveUser.update!(user, otp_recovery_codes: RecoveryCodes.hashed_json(codes))
    user = UserQuery.new.find(user.id)

    RecoveryCodes.consume!(user, codes[0].gsub("-", "")).should be_true
    user = UserQuery.new.find(user.id)
    RecoveryCodes.consume!(user, codes[0]).should be_false
    RecoveryCodes.remaining(user.otp_recovery_codes).should eq(codes.size - 1)
  end
end

describe TwoFactor do
  it "enrolls and confirms with the authenticator code" do
    user = UserFactory.create
    enrollment = TwoFactor.begin_enrollment(user)
    enrollment.codes.size.should eq(8)
    user = UserQuery.new.find(user.id)
    user.otp_enabled.should be_false
    TwoFactor.pending?(user).should be_true

    TwoFactor.confirm(user, Totp.code_at(enrollment.secret, Time.utc)).should be_true
    user = UserQuery.new.find(user.id)
    user.otp_enabled.should be_true
    TwoFactor.authenticate(user, enrollment.codes[0]).should be_true
    user = UserQuery.new.find(user.id)
    TwoFactor.authenticate(user, enrollment.codes[0]).should be_false
    RecoveryCodes.remaining(user.otp_recovery_codes).should eq(7)
  end
end
