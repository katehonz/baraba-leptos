require "http/client"
require "json"
require "base64"

# Line-item OCR for warehouse scans. Separate prompt from accounting MistralDocumentService.
class MistralWarehouseService
  MISTRAL_CHAT_URL = "https://api.mistral.ai/v1/chat/completions"
  MISTRAL_OCR_URL  = "https://api.mistral.ai/v1/ocr"
  VISION_MODEL     = "pixtral-12b-2409"
  OCR_MODEL        = "mistral-ocr-latest"

  class Item
    property name : String?
    property sku : String?
    property barcode : String?
    property quantity : Float64
    property unit : String?
    property unit_price : Float64
    property vat_rate : Float64
    property confidence : Float64?

    def initialize(
      @name : String? = nil,
      @sku : String? = nil,
      @barcode : String? = nil,
      @quantity : Float64 = 1.0,
      @unit : String? = nil,
      @unit_price : Float64 = 0.0,
      @vat_rate : Float64 = 20.0,
      @confidence : Float64? = nil
    )
    end
  end

  class Result
    property vendor_name : String?
    property vendor_vat : String?
    property invoice_number : String?
    property document_date : Time?
    property subtotal : Float64?
    property vat_amount : Float64?
    property total_amount : Float64?
    property confidence : Float64?
    property items : Array(Item)
    property raw_json : String

    def initialize(
      @vendor_name : String? = nil,
      @vendor_vat : String? = nil,
      @invoice_number : String? = nil,
      @document_date : Time? = nil,
      @subtotal : Float64? = nil,
      @vat_amount : Float64? = nil,
      @total_amount : Float64? = nil,
      @confidence : Float64? = nil,
      @items : Array(Item) = [] of Item,
      @raw_json : String = "{}"
    )
    end
  end

  def self.scan(file_data : Bytes, api_key : String, file_name : String, direction : String) : Result
    raise "Моля конфигурирайте Mistral API ключ в настройките на фирмата" if api_key.empty?

    content_type = mime_from_filename(file_name)
    response_body = if content_type == "application/pdf"
                      ocr_text = extract_text_with_ocr(file_data, api_key)
                      call_chat_api(api_key, build_text_payload(ocr_text, direction))
                    elsif content_type.starts_with?("image/")
                      call_chat_api(api_key, build_image_payload(file_data, content_type, direction))
                    else
                      call_chat_api(api_key, build_text_payload(String.new(file_data), direction))
                    end

    result = JSON.parse(response_body)
    content = result["choices"]?.try(&.[0]?).try(&.["message"]?).try(&.["content"]?).try(&.as_s?)
    raise "Празен отговор от Mistral" if content.nil? || content.empty?
    parse_extraction(content, response_body)
  end

  def self.parse_extraction(raw : String, raw_json : String = "{}") : Result
    cleaned = clean_json_block(raw)
    json = begin
      JSON.parse(cleaned)
    rescue
      if extracted = extract_json_object(cleaned)
        JSON.parse(extracted)
      else
        return Result.new(raw_json: raw_json)
      end
    end
    from_json(json, raw_json)
  end

  def self.from_json(json : JSON::Any, raw_json : String = "{}") : Result
    items = [] of Item
    arr = json["items"]?.try(&.as_a?) || json["lines"]?.try(&.as_a?) || [] of JSON::Any
    arr.each do |row|
      name = as_str(row["name"]?) || as_str(row["description"]?)
      items << Item.new(
        name: name,
        sku: as_str(row["sku"]?),
        barcode: as_str(row["barcode"]?),
        quantity: as_f(row["quantity"]?) || 1.0,
        unit: as_str(row["unit"]?),
        unit_price: as_f(row["unitPrice"]?) || as_f(row["unit_price"]?) || 0.0,
        vat_rate: as_f(row["vatRate"]?) || as_f(row["vat_rate"]?) || 20.0,
        confidence: as_f(row["confidence"]?)
      )
    end
    Result.new(
      vendor_name: as_str(json["vendorName"]?) || as_str(json["vendor_name"]?),
      vendor_vat: as_str(json["vendorVat"]?) || as_str(json["vendor_vat"]?),
      invoice_number: as_str(json["invoiceNumber"]?) || as_str(json["invoice_number"]?),
      document_date: parse_date(as_str(json["invoiceDate"]?) || as_str(json["document_date"]?)),
      subtotal: as_f(json["subtotal"]?),
      vat_amount: as_f(json["vatAmount"]?) || as_f(json["vat_amount"]?),
      total_amount: as_f(json["total"]?) || as_f(json["total_amount"]?),
      confidence: as_f(json["confidence"]?),
      items: items,
      raw_json: raw_json
    )
  end

  def self.guess_unit(raw : String?) : String
    s = (raw || "").downcase.strip
    return "C62" if s.empty?
    up = s.upcase
    if SaftUnitOfMeasureQuery.new.code(up).first?
      return up
    end
    return "KGM" if s.includes?("кг") || s == "kg" || s.includes?("килограм")
    return "LTR" if s == "л" || s == "l" || s.includes?("лит")
    return "MTR" if s == "м" || s == "m" || s.includes?("метър")
    return "MTK" if s.includes?("м2") || s.includes?("кв")
    return "MTQ" if s.includes?("м3") || s.includes?("куб")
    return "HUR" if s.includes?("час") || s == "h" || s == "hr"
    return "C62" if s.includes?("бр") || s.includes?("pcs") || s.includes?("бро") || s.includes?("брой")
    "C62"
  end

  private def self.call_chat_api(api_key : String, payload) : String
    uri = URI.parse(MISTRAL_CHAT_URL)
    client = HTTP::Client.new(uri.host.not_nil!, uri.port || 443, tls: true)
    client.read_timeout = 120.seconds
    headers = HTTP::Headers{
      "Authorization" => "Bearer #{api_key}",
      "Content-Type"  => "application/json",
    }
    response = client.post(uri.path.not_nil!, headers: headers, body: payload.to_json)
    raise "Mistral Chat API error: #{response.status_code}" if response.status_code != 200
    response.body
  end

  private def self.extract_text_with_ocr(file_data : Bytes, api_key : String) : String
    uri = URI.parse(MISTRAL_OCR_URL)
    client = HTTP::Client.new(uri.host.not_nil!, uri.port || 443, tls: true)
    client.read_timeout = 120.seconds
    headers = HTTP::Headers{
      "Authorization" => "Bearer #{api_key}",
      "Content-Type"  => "application/json",
    }
    data_url = "data:application/pdf;base64,#{Base64.strict_encode(file_data)}"
    payload = {
      "model"    => OCR_MODEL,
      "document" => {"type" => "document_url", "document_url" => data_url},
    }
    response = client.post(uri.path.not_nil!, headers: headers, body: payload.to_json)
    raise "Mistral OCR API error: #{response.status_code}" if response.status_code != 200
    result = JSON.parse(response.body)
    pages = result["pages"]?.try(&.as_a?) || [] of JSON::Any
    text = pages.map { |page| page["markdown"]?.try(&.as_s?) || "" }.join("\n\n")
    raise "OCR не успя да извлече текст от документа" if text.strip.empty?
    text
  end

  private def self.build_image_payload(file_data : Bytes, content_type : String, direction : String)
    data_url = "data:#{content_type};base64,#{Base64.strict_encode(file_data)}"
    {
      "model"       => VISION_MODEL,
      "messages"    => [
        {
          "role"    => "user",
          "content" => [
            {"type" => "image_url", "image_url" => {"url" => data_url}},
            {"type" => "text", "text" => extraction_prompt(direction)},
          ],
        },
      ],
      "temperature" => 0.1,
      "max_tokens"  => 4000,
    }
  end

  private def self.build_text_payload(text : String, direction : String)
    {
      "model"       => "mistral-small-latest",
      "messages"    => [
        {"role" => "system", "content" => "Ти извличаш редове от складови/търговски документи. Не осчетоводяваш."},
        {"role" => "user", "content" => "#{extraction_prompt(direction)}\n\nТекст:\n#{text}"},
      ],
      "temperature" => 0.1,
      "max_tokens"  => 4000,
    }
  end

  private def self.extraction_prompt(direction : String) : String
    <<-PROMPT
    Извлечи данни от този складов/търговски документ. Посока: #{direction}.
    Върни САМО JSON:
    {
      "vendorName": "",
      "vendorVat": "",
      "invoiceNumber": "",
      "invoiceDate": "YYYY-MM-DD",
      "subtotal": 0,
      "vatAmount": 0,
      "total": 0,
      "confidence": 0.0,
      "items": [
        {"name": "", "sku": "", "barcode": "", "quantity": 1, "unit": "бр", "unitPrice": 0, "vatRate": 20, "confidence": 0.0}
      ]
    }
    unitPrice е единична цена БЕЗ ДДС. quantity е число. Не измисляй редове.
    PROMPT
  end

  private def self.clean_json_block(raw : String) : String
    s = raw.strip
    if s.starts_with?("```")
      s = s.sub(/^```(?:json)?\s*/i, "")
      s = s.sub(/\s*```$/, "")
    end
    s.strip
  end

  private def self.extract_json_object(text : String) : String?
    start = text.index("{")
    finish = text.rindex("}")
    return nil unless start && finish && finish > start
    text[start..finish]
  end

  private def self.parse_date(value : String?) : Time?
    return nil unless value && !value.blank?
    slice = value.size >= 10 ? value[0, 10] : value
    Time.parse(slice, "%Y-%m-%d", Time::Location::UTC)
  rescue
    nil
  end

  private def self.as_str(val : JSON::Any?) : String?
    return nil if val.nil? || val.raw.nil?
    val.as_s? || val.as_i64?.try(&.to_s) || val.as_f?.try(&.to_s)
  end

  private def self.as_f(val : JSON::Any?) : Float64?
    return nil if val.nil? || val.raw.nil?
    val.as_f? || val.as_i64?.try(&.to_f) || val.as_i?.try(&.to_f) || val.as_s?.try(&.gsub(" ", "").gsub(",", ".").to_f?)
  end

  private def self.mime_from_filename(name : String) : String
    lowered = name.downcase
    case
    when lowered.ends_with?(".png")                  then "image/png"
    when lowered.ends_with?(".jpg"), lowered.ends_with?(".jpeg") then "image/jpeg"
    when lowered.ends_with?(".pdf")                  then "application/pdf"
    when lowered.ends_with?(".webp")                 then "image/webp"
    else
      "application/octet-stream"
    end
  end
end
