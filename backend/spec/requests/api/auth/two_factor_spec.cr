require "../../../spec_helper"

describe Api::Auth::TwoFactor::Setup do
  it "returns a QR image and recovery codes after the password" do
    user = verified_user

    response = ApiClient.auth(user).exec_raw(
      Api::Auth::TwoFactor::Setup,
      {password: "password"}.to_json
    )

    response.status_code.should eq(200)
    body = JSON.parse(response.body)
    body["success"].as_bool.should be_true
    body["qr_image"].as_s.should start_with("data:image/svg+xml;base64,")
    body["secret"].as_s.size.should be > 0
    body["recovery_codes"].as_a.size.should eq(8)
  end

  it "rejects a wrong password" do
    user = verified_user

    response = ApiClient.auth(user).exec_raw(
      Api::Auth::TwoFactor::Setup,
      {password: "not-the-password"}.to_json
    )

    response.status_code.should eq(422)
  end
end

describe "signing in with two factor" do
  it "asks for a code and does not accept the challenge token as a session" do
    user = verified_user
    enrollment = TwoFactor.begin_enrollment(user)
    user = UserQuery.new.find(user.id)
    TwoFactor.confirm(user, Totp.code_at(enrollment.secret, Time.utc)).should be_true

    response = ApiClient.new.exec_raw(
      Api::Auth::SignIn::Create,
      {user: {email: user.email, password: "password"}}.to_json
    )
    response.status_code.should eq(200)
    body = JSON.parse(response.body)
    body["requires_2fa"].as_bool.should be_true
    body["token"]?.should be_nil
    challenge = body["challenge_token"].as_s

    me = ApiClient.new.headers("Authorization": "Bearer #{challenge}").exec(Api::Auth::Me)
    me.status_code.should eq(401)

    completed = ApiClient.new.exec_raw(
      Api::Auth::SignIn::Totp,
      {challenge_token: challenge, code: enrollment.codes[0]}.to_json
    )
    completed.status_code.should eq(200)
    token = JSON.parse(completed.body)["token"].as_s

    session = ApiClient.new.headers("Authorization": "Bearer #{token}").exec(Api::Auth::Me)
    session.status_code.should eq(200)
    JSON.parse(session.body)["data"]["otp_enabled"].as_bool.should be_true
  end
end

private def verified_user : User
  user = UserFactory.create
  SaveUser.update!(user, email_verified_at: Time.utc)
  UserQuery.new.find(user.id)
end
