require "http/client"
require "xml"

class Api::ExchangeRates::FetchMonth < ApiAction
  ECB_HIST_90D_URL = "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-hist-90d.xml"
  ECB_HIST_FULL_URL = "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-hist.xml"

  param year : Int32
  param month : Int32

  post "/api/exchange_rates/fetch_month" do
    begin
      # Calculate start and end of month
      start_date = Time.utc(year, month, 1)
      end_date = start_date + 1.month - 1.day

      # Use 90-day history for recent months, full history for older
      ninety_days_ago = Time.utc - 90.days
      url = start_date > ninety_days_ago ? ECB_HIST_90D_URL : ECB_HIST_FULL_URL

      response = HTTP::Client.get(url)

      unless response.status_code == 200
        return json({success: false, error: "Failed to fetch ECB rates: HTTP #{response.status_code}"})
      end

      imported_count, dates = parse_and_import_rates(response.body, start_date, end_date)

      json({
        success:        true,
        message:        "Successfully imported ECB rates for #{year}/#{month}",
        imported_count: imported_count,
        dates:          dates,
      })
    rescue ex
      json({success: false, error: ex.message})
    end
  end

  private def parse_and_import_rates(xml_content : String, start_date : Time, end_date : Time) : Tuple(Int32, Array(String))
    imported_count = 0
    imported_dates = [] of String
    doc = XML.parse(xml_content)

    # Get or create EUR currency
    eur = CurrencyQuery.new.code("EUR").first?
    unless eur
      eur = SaveCurrency.create!(
        code: "EUR",
        name: "Euro",
        name_bg: "Евро",
        symbol: "€",
        decimal_places: 2,
        is_active: true,
        is_base_currency: true
      )
    end

    doc.xpath_nodes("//*[local-name()='Cube' and @time]").each do |time_cube|
      time_str = time_cube["time"]?
      next unless time_str

      date = Time.parse(time_str, "%Y-%m-%d", Time::Location::UTC)

      # Only process dates in the specified month
      next if date < start_date || date > end_date

      date_imported = false

      time_cube.children.each do |node|
        next unless node.element?
        rate_cube = node.as(XML::Node)

        currency_code = rate_cube["currency"]?
        rate_str = rate_cube["rate"]?
        next unless currency_code && rate_str

        rate = rate_str.to_f64
        target_currency = get_or_create_currency(currency_code)

        # Check if rate already exists
        existing = ExchangeRateQuery.new
          .from_currency_id(eur.id)
          .to_currency_id(target_currency.id)
          .valid_date(date)
          .first?

        next if existing

        SaveExchangeRate.create!(
          from_currency_id: eur.id,
          to_currency_id: target_currency.id,
          rate: rate,
          reverse_rate: (1.0 / rate).round(6),
          valid_date: date,
          rate_source: "ecb",
          is_active: true,
          notes: "Imported from ECB"
        )
        imported_count += 1
        date_imported = true
      end

      if date_imported
        imported_dates << date.to_s("%Y-%m-%d")
      end
    end

    {imported_count, imported_dates.sort}
  end

  private def get_or_create_currency(code : String) : Currency
    currency = CurrencyQuery.new.code(code).first?
    return currency if currency

    iso = IsoCurrencyQuery.new.code(code).first?

    if iso
      SaveCurrency.create!(
        code: iso.code,
        name: iso.name,
        name_bg: iso.name_bg,
        symbol: iso.symbol,
        decimal_places: iso.decimal_places,
        is_active: true,
        is_base_currency: false
      )
    else
      SaveCurrency.create!(
        code: code,
        name: code,
        name_bg: nil,
        symbol: nil,
        decimal_places: 2,
        is_active: true,
        is_base_currency: false
      )
    end
  end
end
