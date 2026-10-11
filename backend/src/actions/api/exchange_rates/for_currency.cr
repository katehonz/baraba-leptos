class Api::ExchangeRates::ForCurrency < ApiAction
  include Lucky::SkipRouteStyleCheck

  get "/api/exchange_rates/for_currency" do
    code = params.get?("code").try(&.to_s).try(&.presence)
    date_str = params.get?("date").try(&.to_s).try(&.presence)
    unless code && date_str
      response.status_code = 422
      return json({success: false, error: "code и date са задължителни"})
    end

    parsed = Time.parse(date_str, "%Y-%m-%d", Time::Location::UTC)
    func = params.get?("company_currency").try(&.to_s).try(&.presence) || "EUR"
    data = ExchangeRateLookup.for_currency(code, parsed, func)
    json({success: true, data: data})
  rescue Time::Format::Error
    response.status_code = 422
    json({success: false, error: "Невалиден формат на дата (очакван YYYY-MM-DD)"})
  end
end
