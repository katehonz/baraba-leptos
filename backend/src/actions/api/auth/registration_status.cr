class Api::Auth::RegistrationStatus < ApiAction
  include Api::Auth::SkipRequireAuthToken

  get "/api/auth/registration_status" do
    json({
      success:              true,
      registration_enabled: SystemSetting.registration_enabled?,
    })
  end
end
