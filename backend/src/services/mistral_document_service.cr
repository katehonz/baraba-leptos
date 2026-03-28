require "http/client"
require "json"
require "base64"

# Mistral AI service for invoice scanning (replaces Azure Document Intelligence)
class MistralDocumentService
  MISTRAL_CHAT_URL = "https://api.mistral.ai/v1/chat/completions"
  MISTRAL_OCR_URL  = "https://api.mistral.ai/v1/ocr"
  VISION_MODEL     = "pixtral-12b-2409"  # Vision model for images
  OCR_MODEL        = "mistral-ocr-latest" # OCR model for PDFs

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

  def self.scan_invoice(file_data : Bytes, api_key : String, direction : String, file_name : String = "invoice.pdf") : RecognitionResult
    if api_key.empty?
      raise "Моля конфигурирайте Mistral API ключ в настройките на фирмата"
    end

    begin
      content_type = mime_from_filename(file_name)

      # For PDFs, use OCR API first to extract text
      if content_type == "application/pdf"
        ocr_text = extract_text_with_ocr(file_data, api_key)
        response_body = call_chat_api(api_key, build_text_payload(ocr_text, direction))
      elsif content_type.starts_with?("image/")
        # For images, use Pixtral vision model directly
        response_body = call_chat_api(api_key, build_image_payload(file_data, content_type, direction))
      else
        # Try as text
        text = String.new(file_data)
        response_body = call_chat_api(api_key, build_text_payload(text, direction))
      end

      result = JSON.parse(response_body)
      content = result["choices"]?.try(&.[0]?).try(&.["message"]?).try(&.["content"]?).try(&.as_s?)

      if content.nil? || content.empty?
        raise "Празен отговор от Mistral"
      end

      parsed_invoice = parse_extraction(content, direction)
      RecognitionResult.new(parsed_invoice, response_body)

    rescue ex
      Log.error { "Mistral Document Service error: #{ex.message}" }
      raise ex
    end
  end

  # Call Mistral Chat API
  private def self.call_chat_api(api_key : String, payload) : String
    uri = URI.parse(MISTRAL_CHAT_URL)
    client = HTTP::Client.new(uri.host.not_nil!, uri.port || 443, tls: true)
    client.read_timeout = 120.seconds

    headers = HTTP::Headers{
      "Authorization" => "Bearer #{api_key}",
      "Content-Type"  => "application/json",
    }

    response = client.post(uri.path.not_nil!, headers: headers, body: payload.to_json)

    if response.status_code != 200
      raise "Mistral Chat API error: #{response.status_code} - #{response.body}"
    end

    response.body
  end

  # Extract text from PDF using Mistral OCR API
  private def self.extract_text_with_ocr(file_data : Bytes, api_key : String) : String
    uri = URI.parse(MISTRAL_OCR_URL)
    client = HTTP::Client.new(uri.host.not_nil!, uri.port || 443, tls: true)
    client.read_timeout = 120.seconds

    headers = HTTP::Headers{
      "Authorization" => "Bearer #{api_key}",
      "Content-Type"  => "application/json",
    }

    # Encode PDF as base64 data URL
    base64_data = Base64.strict_encode(file_data)
    data_url = "data:application/pdf;base64,#{base64_data}"

    payload = {
      "model" => OCR_MODEL,
      "document" => {
        "type" => "document_url",
        "document_url" => data_url
      }
    }

    response = client.post(uri.path.not_nil!, headers: headers, body: payload.to_json)

    if response.status_code != 200
      raise "Mistral OCR API error: #{response.status_code} - #{response.body}"
    end

    result = JSON.parse(response.body)

    # Extract markdown text from all pages
    pages = result["pages"]?.try(&.as_a?) || [] of JSON::Any
    text_parts = pages.map do |page|
      page["markdown"]?.try(&.as_s?) || ""
    end

    extracted_text = text_parts.join("\n\n")

    if extracted_text.strip.empty?
      raise "OCR не успя да извлече текст от PDF документа"
    end

    extracted_text
  end

  private def self.build_image_payload(file_data : Bytes, content_type : String, direction : String)
    data_url = "data:#{content_type};base64,#{Base64.strict_encode(file_data)}"

    {
      "model" => VISION_MODEL,
      "messages" => [
        {
          "role" => "user",
          "content" => [
            {
              "type" => "image_url",
              "image_url" => {"url" => data_url}
            },
            {
              "type" => "text",
              "text" => extraction_prompt(direction)
            }
          ]
        }
      ],
      "temperature" => 0.1,
      "max_tokens" => 2000
    }
  end

  private def self.build_text_payload(text : String, direction : String)
    {
      "model" => "mistral-small-latest",
      "messages" => [
        {"role" => "system", "content" => "Ти си експерт по български счетоводни документи."},
        {"role" => "user", "content" => "#{extraction_prompt(direction)}\n\nТекст на документа:\n#{text}"}
      ],
      "temperature" => 0.1,
      "max_tokens" => 2000
    }
  end

  private def self.extraction_prompt(direction : String) : String
    direction_hint = direction == "sale" ? "SALE (Продажба)" : "PURCHASE (Покупка)"
    counterpart_hint = if direction == "sale"
      "КЛИЕНТЪТ/ПОЛУЧАТЕЛЯТ - фирмата НА КОЯТО издаваме фактурата (НЕ издателя!)"
    else
      "ДОСТАВЧИКЪТ/ИЗДАТЕЛЯТ - фирмата ОТ КОЯТО получаваме фактурата"
    end

    <<-PROMPT
    Анализирай тази българска фактура и върни САМО валиден JSON.

    ⚠️ ПОТРЕБИТЕЛЯТ УКАЗА: Това е #{direction_hint}

    Структура на JSON:
    {
      "documentType": "INVOICE" | "CREDIT_NOTE" | "DEBIT_NOTE",
      "transactionType": "#{direction == "sale" ? "SALE" : "PURCHASE"}",
      "documentNumber": "номер на фактурата",
      "documentDate": "YYYY-MM-DD",
      "dueDate": "YYYY-MM-DD",
      "counterpart": {
        "name": "име на контрагента",
        "eik": "ЕИК (9-13 цифри)",
        "vatNumber": "ДДС номер (BG + цифри)",
        "address": "адрес"
      },
      "netAmount": число,
      "vatAmount": число,
      "totalAmount": число
    }

    ⚠️⚠️⚠️ КРИТИЧНО ЗА counterpart:
    counterpart трябва да е: #{counterpart_hint}

    #{direction == "sale" ? "За ПРОДАЖБА: Търси секция \"ПОЛУЧАТЕЛ\", \"КУПУВАЧ\", \"КЛИЕНТ\" - това е counterpart!" : "За ПОКУПКА: Търси секция \"ДОСТАВЧИК\", \"ИЗДАТЕЛ\", \"ПРОДАВАЧ\" - това е counterpart!"}

    ⚠️ НЕ връщай данните на ИЗДАТЕЛЯ/ДОСТАВЧИКА ако това е продажба!
    ⚠️ Издателят на фактурата е НАШАТА фирма при продажба!

    Правила за идентификатори:
    - vatNumber: ЗАДЪЛЖИТЕЛНО с префикс BG (напр. BG000000000)
    - eik: САМО цифри, 9 или 13 знака, БЕЗ BG

    Отговори САМО с JSON без пояснения.
    PROMPT
  end

  private def self.parse_extraction(raw : String, direction : String) : ScannedInvoiceData
    # Clean JSON from markdown code blocks
    cleaned = clean_json_block(raw)

    begin
      json = JSON.parse(cleaned)
    rescue
      # Try to extract JSON object from text
      if extracted = extract_json_object(cleaned)
        json = JSON.parse(extracted)
      else
        return create_empty_invoice(direction, "Неуспешно парсиране на JSON от Mistral")
      end
    end

    invoice = ScannedInvoiceData.new(direction)

    # Use user's selected direction - they know better what they're scanning
    # AI's transactionType is just informational
    actual_direction = direction
    invoice.direction = actual_direction

    invoice.invoice_number = json["documentNumber"]?.try(&.as_s?)
    invoice.invoice_date = parse_date(json["documentDate"]?.try(&.as_s?))
    invoice.due_date = parse_date(json["dueDate"]?.try(&.as_s?))
    invoice.subtotal = json["netAmount"]?.try(&.as_f?) || json["netAmount"]?.try(&.as_i64?).try(&.to_f64)
    invoice.total_tax = json["vatAmount"]?.try(&.as_f?) || json["vatAmount"]?.try(&.as_i64?).try(&.to_f64)
    invoice.invoice_total = json["totalAmount"]?.try(&.as_f?) || json["totalAmount"]?.try(&.as_i64?).try(&.to_f64)

    # Extract counterpart info
    if counterpart = json["counterpart"]?
      cp_name = counterpart["name"]?.try(&.as_s?)
      cp_vat = counterpart["vatNumber"]?.try(&.as_s?)
      cp_eik = counterpart["eik"]?.try(&.as_s?)
      cp_address = counterpart["address"]?.try(&.as_s?)

      # Normalize VAT number
      if cp_vat && !cp_vat.empty?
        cp_vat = cp_vat.strip.upcase
        cp_vat = "BG#{cp_vat}" unless cp_vat.starts_with?("BG")
      end

      # For purchases, counterpart is vendor; for sales, counterpart is customer
      if actual_direction == "purchase"
        invoice.vendor_name = cp_name
        invoice.vendor_vat_number = cp_vat
        invoice.vendor_address = cp_address
      else
        invoice.customer_name = cp_name
        invoice.customer_vat_number = cp_vat
        invoice.customer_address = cp_address
      end
    end

    # Determine if manual review is needed
    missing_fields = [] of String
    missing_fields << "номер" if invoice.invoice_number.nil? || invoice.invoice_number.try(&.empty?)
    missing_fields << "дата" if invoice.invoice_date.nil?
    missing_fields << "сума" if invoice.invoice_total.nil?

    vat_number = actual_direction == "purchase" ? invoice.vendor_vat_number : invoice.customer_vat_number
    missing_fields << "ДДС номер на контрагент" if vat_number.nil? || vat_number.try(&.empty?)

    if !missing_fields.empty?
      invoice.requires_manual_review = true
      invoice.manual_review_reason = "Липсват: #{missing_fields.join(", ")}"
      invoice.confidence = 0.5
    else
      invoice.confidence = 0.9
    end

    invoice
  end

  private def self.clean_json_block(input : String) : String
    trimmed = input.strip

    # Remove markdown code blocks
    if trimmed.starts_with?("```json")
      trimmed = trimmed[7..]
    elsif trimmed.starts_with?("```")
      trimmed = trimmed[3..]
    end

    if trimmed.ends_with?("```")
      trimmed = trimmed[0...-3]
    end

    trimmed.strip
  end

  private def self.extract_json_object(input : String) : String?
    brace_depth = 0
    start_index : Int32? = nil
    in_string = false
    is_escaped = false

    input.each_char_with_index do |ch, idx|
      if ch == '\\' && in_string
        is_escaped = !is_escaped
        next
      end

      if ch == '"' && !is_escaped
        in_string = !in_string
      end
      is_escaped = false

      next if in_string

      case ch
      when '{'
        start_index = idx if brace_depth == 0
        brace_depth += 1
      when '}'
        if brace_depth > 0
          brace_depth -= 1
          if brace_depth == 0 && start_index
            return input[start_index..idx]
          end
        end
      end
    end

    nil
  end

  private def self.parse_date(date_str : String?) : Time?
    return nil if date_str.nil? || date_str.empty?

    formats = ["%Y-%m-%d", "%d.%m.%Y", "%d/%m/%Y"]
    formats.each do |fmt|
      begin
        return Time.parse(date_str, fmt, Time::Location::UTC)
      rescue
        next
      end
    end
    nil
  end

  private def self.mime_from_filename(name : String) : String
    lowered = name.downcase
    case
    when lowered.ends_with?(".png")  then "image/png"
    when lowered.ends_with?(".jpg"), lowered.ends_with?(".jpeg") then "image/jpeg"
    when lowered.ends_with?(".pdf")  then "application/pdf"
    when lowered.ends_with?(".webp") then "image/webp"
    else "application/octet-stream"
    end
  end

  private def self.create_empty_invoice(direction : String, reason : String? = nil) : ScannedInvoiceData
    invoice = ScannedInvoiceData.new(direction)
    invoice.confidence = 0.0
    invoice.requires_manual_review = true
    invoice.manual_review_reason = reason || "Не са разпознати данни от фактурата"
    invoice
  end
end
