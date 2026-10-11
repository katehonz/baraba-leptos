require "../../../spec_helper"

describe Api::ExchangeRates::ForCurrency do
  it "returns identity for matching currencies" do
    user = UserFactory.create &.is_super_admin(true)
    response = ApiClient.auth(user).get(
      "/api/exchange_rates/for_currency?code=EUR&date=2026-09-12&company_currency=EUR"
    )
    response.status_code.should eq(200)
    json = JSON.parse(response.body)
    json["success"].as_bool.should be_true
    json["data"]["found"].as_bool.should be_true
    json["data"]["exchange_rate"].as_f.should eq(1.0)
    json["data"]["source"].as_s.should eq("identity")
  end

  it "returns the fixed BGN to EUR rate" do
    user = UserFactory.create &.is_super_admin(true)
    response = ApiClient.auth(user).get(
      "/api/exchange_rates/for_currency?code=BGN&date=2026-09-12&company_currency=EUR"
    )
    response.status_code.should eq(200)
    json = JSON.parse(response.body)
    json["data"]["source"].as_s.should eq("fixed")
    json["data"]["ecb_rate"].as_f.should eq(1.95583)
  end
end
