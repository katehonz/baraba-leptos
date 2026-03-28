require "http/client"
require "json"
require "base64"

# Azure Document Intelligence service for invoice scanning
class AzureDocumentService
  PREBUILT_INVOICE_MODEL = "prebuilt-invoice"
  API_VERSION            = "2023-07-31"

  class RecognitionResult
    property invoice : ScannedInvoiceData
    property raw_json : String

    def initialize(@invoice : ScannedInvoiceData, @raw_json : String)
    end
  end

  class ScannedInvoiceData
    property direction : String
    property vendor_name : String?
    property vendor_vat_number : String?
    property vendor_address : String?
    property customer_name : String?
    property customer_vat_number : String?
    property customer_address : String?
    property invoice_number : String?
    property invoice_date : Time?
    property due_date : Time?
    property subtotal : Float64?
    property total_tax : Float64?
    property invoice_total : Float64?
    property confidence : Float64?
    property requires_manual_review : Bool
    property manual_review_reason : String?

    def initialize(
      @direction : String,
      @vendor_name : String? = nil,
      @vendor_vat_number : String? = nil,
      @vendor_address : String? = nil,
      @customer_name : String? = nil,
      @customer_vat_number : String? = nil,
      @customer_address : String? = nil,
      @invoice_number : String? = nil,
      @invoice_date : Time? = nil,
      @due_date : Time? = nil,
      @subtotal : Float64? = nil,
      @total_tax : Float64? = nil,
      @invoice_total : Float64? = nil,
      @confidence : Float64? = nil,
      @requires_manual_review : Bool = false,
      @manual_review_reason : String? = nil
    )
    end
  end

  def self.scan_invoice(file_data : Bytes, endpoint : String, api_key : String, direction : String) : RecognitionResult
    if endpoint.empty? || api_key.empty?
      return create_mock_invoice(direction)
    end

    begin
      # Build Azure DI URL
      uri = URI.parse("#{endpoint}/formrecognizer/documentModels/#{PREBUILT_INVOICE_MODEL}:analyze?api-version=#{API_VERSION}")

      # Create HTTPS client
      client = HTTP::Client.new(uri.host.not_nil!, uri.port || 443, tls: true)
      client.read_timeout = 120.seconds

      # Call Azure Document Intelligence
      headers = HTTP::Headers{
        "Ocp-Apim-Subscription-Key" => api_key,
        "Content-Type"              => "application/octet-stream",
      }

      response = client.post(uri.path.not_nil! + "?" + uri.query.not_nil!, headers: headers, body: file_data)

      if response.status_code != 202
        raise "Azure API error: #{response.status_code} - #{response.body}"
      end

      # Get operation location for polling
      operation_location = response.headers["Operation-Location"]?
      if operation_location.nil?
        raise "No Operation-Location header in response"
      end

      # Poll for result
      result_json = poll_for_result(operation_location, api_key)
      parsed_result = parse_azure_result(result_json, direction)

      RecognitionResult.new(parsed_result, result_json)
    rescue ex
      Log.error { "Azure Document Intelligence error: #{ex.message}" }
      raise ex
    end
  end

  private def self.poll_for_result(operation_url : String, api_key : String) : String
    uri = URI.parse(operation_url)
    client = HTTP::Client.new(uri.host.not_nil!, uri.port || 443, tls: true)
    client.read_timeout = 30.seconds

    headers = HTTP::Headers{
      "Ocp-Apim-Subscription-Key" => api_key,
    }

    30.times do
      response = client.get(uri.path.not_nil! + "?" + uri.query.to_s, headers: headers)

      if response.status_code != 200
        raise "Failed to get Azure result: #{response.status_code}"
      end

      result = JSON.parse(response.body)
      status = result["status"]?.try(&.as_s?)

      case status
      when "succeeded"
        return response.body
      when "failed"
        raise "Azure analysis failed: #{result["error"]?}"
      when "running", "notStarted"
        sleep 2.seconds
      else
        sleep 2.seconds
      end
    end

    raise "Azure analysis timeout"
  end

  private def self.parse_azure_result(json_str : String, direction : String) : ScannedInvoiceData
    result = JSON.parse(json_str)

    analyze_result = result["analyzeResult"]?
    return create_empty_invoice(direction) if analyze_result.nil?

    documents = analyze_result["documents"]?.try(&.as_a?)
    return create_empty_invoice(direction) if documents.nil? || documents.empty?

    doc = documents.first
    fields = doc["fields"]?
    confidence = doc["confidence"]?.try(&.as_f?) || 0.0

    return create_empty_invoice(direction) if fields.nil?

    invoice = ScannedInvoiceData.new(direction)
    invoice.vendor_name = get_string_value(fields, "VendorName")
    invoice.vendor_vat_number = get_string_value(fields, "VendorTaxId")
    invoice.vendor_address = get_address_value(fields, "VendorAddress")
    invoice.customer_name = get_string_value(fields, "CustomerName")
    invoice.customer_vat_number = get_string_value(fields, "CustomerTaxId")
    invoice.customer_address = get_address_value(fields, "CustomerAddress")
    invoice.invoice_number = get_string_value(fields, "InvoiceId")
    invoice.invoice_date = get_date_value(fields, "InvoiceDate")
    invoice.due_date = get_date_value(fields, "DueDate")
    invoice.subtotal = get_currency_value(fields, "SubTotal")
    invoice.total_tax = get_currency_value(fields, "TotalTax")
    invoice.invoice_total = get_currency_value(fields, "InvoiceTotal")
    invoice.confidence = confidence
    invoice.requires_manual_review = confidence < 0.8
    invoice.manual_review_reason = confidence < 0.8 ? "Ниска увереност: #{(confidence * 100).round(1)}%" : nil
    invoice
  end

  private def self.get_string_value(fields : JSON::Any, field_name : String) : String?
    field = fields[field_name]?
    return nil if field.nil?

    # Try valueString first
    value = field["valueString"]?.try(&.as_s?)
    return value if value

    # Fallback to content
    field["content"]?.try(&.as_s?)
  end

  private def self.get_address_value(fields : JSON::Any, field_name : String) : String?
    field = fields[field_name]?
    return nil if field.nil?

    addr = field["valueAddress"]?
    if addr
      parts = [] of String
      if val = addr["streetAddress"]?.try(&.as_s?)
        parts << val
      end
      if val = addr["city"]?.try(&.as_s?)
        parts << val
      end
      if val = addr["postalCode"]?.try(&.as_s?)
        parts << val
      end
      if val = addr["countryRegion"]?.try(&.as_s?)
        parts << val
      end
      return parts.join(", ") unless parts.empty?
    end

    # Fallback to content
    field["content"]?.try(&.as_s?)
  end

  private def self.get_date_value(fields : JSON::Any, field_name : String) : Time?
    field = fields[field_name]?
    return nil if field.nil?

    date_str = field["valueDate"]?.try(&.as_s?)
    return nil if date_str.nil?

    begin
      Time.parse(date_str, "%Y-%m-%d", Time::Location::UTC)
    rescue
      nil
    end
  end

  private def self.get_currency_value(fields : JSON::Any, field_name : String) : Float64?
    field = fields[field_name]?
    return nil if field.nil?

    # Try valueCurrency first
    currency = field["valueCurrency"]?
    if currency
      return currency["amount"]?.try(&.as_f?)
    end

    # Try valueNumber
    field["valueNumber"]?.try(&.as_f?)
  end

  private def self.create_mock_invoice(direction : String) : RecognitionResult
    invoice = ScannedInvoiceData.new(direction)
    invoice.vendor_name = "Тестов Доставчик ЕООД"
    invoice.vendor_vat_number = "BG000000000"
    invoice.vendor_address = "София, България"
    invoice.customer_name = "Тестов Клиент ООД"
    invoice.customer_vat_number = "BG000000000"
    invoice.customer_address = "Пловдив, България"
    invoice.invoice_number = "INV-2025-001"
    invoice.invoice_date = Time.utc
    invoice.due_date = Time.utc + 30.days
    invoice.subtotal = 1000.0
    invoice.total_tax = 200.0
    invoice.invoice_total = 1200.0
    invoice.confidence = 0.95
    invoice.requires_manual_review = false
    invoice.manual_review_reason = "Тестови данни - Azure не е конфигуриран"

    RecognitionResult.new(invoice, %({"mock": true, "reason": "Azure credentials not configured"}))
  end

  private def self.create_empty_invoice(direction : String) : ScannedInvoiceData
    invoice = ScannedInvoiceData.new(direction)
    invoice.confidence = 0.0
    invoice.requires_manual_review = true
    invoice.manual_review_reason = "Не са разпознати данни от фактурата"
    invoice
  end
end
