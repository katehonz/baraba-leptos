require "http/client"
require "xml"

class Api::ExchangeRates::FetchLatest < ApiAction
  ECB_DAILY_URL = "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml"

  post "/api/exchange_rates/fetch_latest" do
    begin
      # Fetch XML from ECB
      response = HTTP::Client.get(ECB_DAILY_URL)

      unless response.status_code == 200
        return json({success: false, error: "Failed to fetch ECB rates: HTTP #{response.status_code}"})
      end

      # Parse and import rates
      imported_count = parse_and_import_rates(response.body)

      json({
        success:        true,
        message:        "Successfully imported ECB daily rates",
        imported_count: imported_count,
      })
    rescue ex
      json({success: false, error: ex.message})
    end
  end

  private def parse_and_import_rates(xml_content : String) : Int32
    imported_count = 0
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

    # Parse ECB XML - structure: //Cube[@time]/Cube[@currency]
    # Use local-name() to handle namespaces
    doc.xpath_nodes("//*[local-name()='Cube' and @time]").each do |time_cube|
      time_str = time_cube["time"]?
      next unless time_str

      date = Time.parse(time_str, "%Y-%m-%d", Time::Location::UTC)

      time_cube.children.each do |node|
        next unless node.element?
        rate_cube = node.as(XML::Node)

        currency_code = rate_cube["currency"]?
        rate_str = rate_cube["rate"]?
        next unless currency_code && rate_str

        rate = rate_str.to_f64

        # Get or create target currency
        target_currency = get_or_create_currency(currency_code)

        # Check if rate already exists
        existing = ExchangeRateQuery.new
          .from_currency_id(eur.id)
          .to_currency_id(target_currency.id)
          .first?

        if existing
          # Check if same date
          if existing.valid_date.to_s("%Y-%m-%d") == date.to_s("%Y-%m-%d")
            next # Skip duplicate
          end
        end

        # Create exchange rate
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
      end

      # Only process the first (latest) date for daily rates
      break
    end

    imported_count
  end

  private def get_or_create_currency(code : String) : Currency
    currency = CurrencyQuery.new.code(code).first?
    return currency if currency

    # Try to get info from ISO currencies
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
