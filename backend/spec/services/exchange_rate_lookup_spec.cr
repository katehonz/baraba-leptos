require "../spec_helper"

describe ExchangeRateLookup do
  it "returns identity when document and company currencies match" do
    r = ExchangeRateLookup.for_currency("EUR", Time.utc(2026, 9, 12), "EUR")
    r[:found].should be_true
    r[:source].should eq("identity")
    r[:exchange_rate].should eq(1.0)
    r[:ecb_rate].should eq(1.0)
  end

  it "uses the fixed BGN/EUR rate toward EUR" do
    r = ExchangeRateLookup.for_currency("BGN", Time.utc(2026, 9, 12), "EUR")
    r[:found].should be_true
    r[:source].should eq("fixed")
    r[:ecb_rate].should eq(1.95583)
    r[:exchange_rate].should be_close(1.0 / 1.95583, 1e-9)
  end

  it "uses the fixed BGN/EUR rate toward BGN" do
    r = ExchangeRateLookup.for_currency("EUR", Time.utc(2026, 9, 12), "BGN")
    r[:found].should be_true
    r[:source].should eq("fixed")
    r[:exchange_rate].should eq(1.95583)
  end

  it "converts via the latest ECB quote on or before the date" do
    eur = SaveCurrency.create!(
      code: "EUR", name: "Euro", name_bg: "Евро", symbol: "€",
      decimal_places: 2, is_active: true, is_base_currency: true
    )
    usd = SaveCurrency.create!(
      code: "USD", name: "US Dollar", name_bg: "Щатски долар", symbol: "$",
      decimal_places: 2, is_active: true, is_base_currency: false
    )
    SaveExchangeRate.create!(
      from_currency_id: eur.id,
      to_currency_id: usd.id,
      rate: 1.10,
      reverse_rate: 1.0 / 1.10,
      valid_date: Time.utc(2026, 9, 10),
      rate_source: "ecb",
      is_active: true
    )

    r = ExchangeRateLookup.for_currency("USD", Time.utc(2026, 9, 12), "EUR")
    r[:found].should be_true
    r[:source].should eq("ecb")
    r[:ecb_rate].should eq(1.10)
    r[:exchange_rate].should be_close(1.0 / 1.10, 1e-9)
    r[:rate_date].should eq("2026-09-10")
  end

  it "returns missing when no ECB row exists" do
    r = ExchangeRateLookup.for_currency("JPY", Time.utc(2026, 9, 12), "EUR")
    r[:found].should be_false
    r[:source].should eq("missing")
  end
end
