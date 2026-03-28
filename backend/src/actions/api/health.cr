# Health check endpoint for Docker healthcheck and load balancers
class Api::Health < ApiAction
  include Api::Auth::SkipRequireAuthToken

  get "/api/health" do
    json({status: "ok", timestamp: Time.utc.to_rfc3339})
  end
end
