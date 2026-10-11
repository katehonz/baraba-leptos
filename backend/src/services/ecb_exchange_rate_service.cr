require "http/client"
require "xml"

# Service for fetching exchange rates from the European Central Bank (ECB)
# ECB publishes daily reference rates for major currencies against EUR
class EcbExchangeRateService
  ECB_DAILY_URL = "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml"
  ECB_HIST_URL  = "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-hist.xml"

  struct RateData
    getter currency : String
    getter rate : Float64
    getter date : Time

    def initialize(@currency : String, @rate : Float64, @date : Time)
    end
  end

  # Fetch latest exchange rates from ECB
  def self.fetch_latest : Array(RateData)
    response = HTTP::Client.get(ECB_DAILY_URL)

    if response.status_code == 200
      parse_ecb_xml(response.body)
    else
      raise "Failed to fetch ECB rates: HTTP #{response.status_code}"
    end
  end

  # Fetch rates for a specific date from ECB historical data
  def self.fetch_for_date(date : Time) : Array(RateData)
    response = HTTP::Client.get(ECB_HIST_URL)

    if response.status_code == 200
      parse_ecb_xml(response.body, date)
    else
      raise "Failed to fetch ECB historical rates: HTTP #{response.status_code}"
    end
  end

  # Fetch all rates for a specific month
  def self.fetch_for_month(year : Int32, month : Int32) : Array(RateData)
    response = HTTP::Client.get(ECB_HIST_URL)

    if response.status_code == 200
      parse_ecb_xml_month(response.body, year, month)
    else
      raise "Failed to fetch ECB historical rates: HTTP #{response.status_code}"
    end
  end

  # Parse ECB XML response
  private def self.parse_ecb_xml(xml_content : String, filter_date : Time? = nil) : Array(RateData)
    rates = [] of RateData
    doc = XML.parse(xml_content)

    # ECB XML structure:
    # <gesmes:Envelope>
    #   <Cube>
    #     <Cube time="2024-01-26">
    #       <Cube currency="USD" rate="1.0856"/>
    #       ...
    #     </Cube>
    #   </Cube>
    # </gesmes:Envelope>

    doc.xpath_nodes("//Cube[@time]").each do |time_cube|
      time_str = time_cube["time"]?
      next unless time_str

      date = Time.parse(time_str, "%Y-%m-%d", Time::Location::UTC)

      # If filtering by date, only process matching date
      if filter_date
        next unless date.year == filter_date.year &&
                    date.month == filter_date.month &&
                    date.day == filter_date.day
      end

      time_cube.xpath_nodes("Cube[@currency]").each do |rate_cube|
        currency = rate_cube["currency"]?
        rate_str = rate_cube["rate"]?

        next unless currency && rate_str

        rate = rate_str.to_f64
        rates << RateData.new(currency, rate, date)
      end

      # For daily rates, we only need the first (latest) date
      break unless filter_date
    end

    rates
  end

  # Parse ECB XML and get all rates for a specific month
  private def self.parse_ecb_xml_month(xml_content : String, year : Int32, month : Int32) : Array(RateData)
    rates = [] of RateData
    doc = XML.parse(xml_content)

    doc.xpath_nodes("//Cube[@time]").each do |time_cube|
      time_str = time_cube["time"]?
      next unless time_str

      date = Time.parse(time_str, "%Y-%m-%d", Time::Location::UTC)

      # Only process dates in the specified month
      next unless date.year == year && date.month == month

      time_cube.xpath_nodes("Cube[@currency]").each do |rate_cube|
        currency = rate_cube["currency"]?
        rate_str = rate_cube["rate"]?

        next unless currency && rate_str

        rate = rate_str.to_f64
        rates << RateData.new(currency, rate, date)
      end
    end

    rates
  end

  # Import rates into database
  def self.import_rates(rates : Array(RateData), user_id : Int64? = nil) : Int32
    imported_count = 0

    # Get EUR currency (base currency)
    eur = CurrencyQuery.new.code("EUR").first?
    unless eur
      # Create EUR if it doesn't exist
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

    rates.each do |rate_data|
      # Find or create the target currency
      target_currency = CurrencyQuery.new.code(rate_data.currency).first?

      unless target_currency
        # Try to get from ISO currencies
        iso_currency = IsoCurrencyQuery.new.code(rate_data.currency).first?

        if iso_currency
          target_currency = SaveCurrency.create!(
            code: iso_currency.code,
            name: iso_currency.name,
            name_bg: iso_currency.name_bg,
            symbol: iso_currency.symbol,
            decimal_places: iso_currency.decimal_places,
            is_active: true,
            is_base_currency: false
          )
        else
          # Create with basic info
          target_currency = SaveCurrency.create!(
            code: rate_data.currency,
            name: rate_data.currency,
            name_bg: nil,
            symbol: nil,
            decimal_places: 2,
            is_active: true,
            is_base_currency: false
          )
        end
      end

      # Check if rate already exists for this date and currency pair
      existing = ExchangeRateQuery.new
        .from_currency_id(eur.id)
        .to_currency_id(target_currency.id)
        .valid_date(rate_data.date)
        .first?

      unless existing
        # Create the exchange rate
        SaveExchangeRate.create!(
          from_currency_id: eur.id,
          to_currency_id: target_currency.id,
          rate: rate_data.rate,
          reverse_rate: (1.0 / rate_data.rate).round(6),
          valid_date: rate_data.date,
          rate_source: "ecb",
          is_active: true,
          notes: "Imported from ECB",
          created_by_id: user_id
        )
        imported_count += 1
      end
    end

    imported_count
  end
end
