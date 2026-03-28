# Техническа документация: Mistral AI OCR интеграция

## Съдържание

1. [Общ преглед](#общ-преглед)
2. [Архитектура](#архитектура)
3. [Mistral API endpoints](#mistral-api-endpoints)
4. [Имплементация](#имплементация)
5. [Формат на заявките](#формат-на-заявките)
6. [Обработка на отговорите](#обработка-на-отговорите)
7. [Prompt Engineering](#prompt-engineering)
8. [Конфигурация](#конфигурация)
9. [Примерен код](#примерен-код)

---

## Общ преглед

Тази интеграция използва Mistral AI за автоматично разпознаване на български фактури. Поддържат се два типа документи:

| Тип файл | Модел | API |
|----------|-------|-----|
| PDF | `mistral-ocr-latest` + `mistral-small-latest` | OCR API → Chat API |
| Изображения (PNG, JPG) | `pixtral-12b-2409` | Chat API (vision) |

### Процес на обработка

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Upload    │────▶│   Detect    │────▶│   Process   │────▶│   Parse     │
│   File      │     │   Type      │     │   with AI   │     │   JSON      │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
                           │                   │
                           ▼                   ▼
                    ┌─────────────┐     ┌─────────────┐
                    │    PDF?     │     │   Image?    │
                    │  OCR + Chat │     │   Vision    │
                    └─────────────┘     └─────────────┘
```

---

## Архитектура

### Компоненти

```
backend/
├── src/
│   ├── services/
│   │   └── mistral_document_service.cr    # Основен сервиз
│   └── actions/api/scanned_invoices/
│       └── scan.cr                         # API endpoint
│
leptos/
└── src/pages/
    └── scanned_invoices.rs                 # Frontend UI
```

### Поток на данните

1. **Frontend** → Base64 кодира файла → POST към API
2. **Backend API** → Валидира → Извиква MistralDocumentService
3. **MistralDocumentService** → Определя типа на файла → Извиква Mistral API
4. **Mistral API** → Връща JSON с извлечени данни
5. **Backend** → Парсва JSON → Валидира с VIES → Записва в БД

---

## Mistral API endpoints

### 1. OCR API (за PDF файлове)

**URL:** `https://api.mistral.ai/v1/ocr`

**Модел:** `mistral-ocr-latest`

**Заявка:**
```json
{
  "model": "mistral-ocr-latest",
  "document": {
    "type": "document_url",
    "document_url": "data:application/pdf;base64,{BASE64_ENCODED_PDF}"
  }
}
```

**Отговор:**
```json
{
  "pages": [
    {
      "index": 0,
      "markdown": "# ФАКТУРА\n\nНомер: 0000001234\nДата: 15.01.2024\n..."
    }
  ]
}
```

### 2. Chat API (за анализ на текст)

**URL:** `https://api.mistral.ai/v1/chat/completions`

**Модел за текст:** `mistral-small-latest`

**Заявка:**
```json
{
  "model": "mistral-small-latest",
  "messages": [
    {
      "role": "system",
      "content": "Ти си експерт по български счетоводни документи."
    },
    {
      "role": "user",
      "content": "{EXTRACTION_PROMPT}\n\nТекст на документа:\n{OCR_TEXT}"
    }
  ],
  "temperature": 0.1,
  "max_tokens": 2000
}
```

### 3. Vision API (за изображения)

**URL:** `https://api.mistral.ai/v1/chat/completions`

**Модел:** `pixtral-12b-2409`

**Заявка:**
```json
{
  "model": "pixtral-12b-2409",
  "messages": [
    {
      "role": "user",
      "content": [
        {
          "type": "image_url",
          "image_url": {
            "url": "data:image/png;base64,{BASE64_ENCODED_IMAGE}"
          }
        },
        {
          "type": "text",
          "text": "{EXTRACTION_PROMPT}"
        }
      ]
    }
  ],
  "temperature": 0.1,
  "max_tokens": 2000
}
```

---

## Имплементация

### Основен клас: MistralDocumentService

```crystal
class MistralDocumentService
  MISTRAL_CHAT_URL = "https://api.mistral.ai/v1/chat/completions"
  MISTRAL_OCR_URL  = "https://api.mistral.ai/v1/ocr"
  VISION_MODEL     = "pixtral-12b-2409"
  OCR_MODEL        = "mistral-ocr-latest"

  def self.scan_invoice(
    file_data : Bytes,
    api_key : String,
    direction : String,
    file_name : String
  ) : RecognitionResult
    content_type = mime_from_filename(file_name)

    if content_type == "application/pdf"
      # PDF: OCR → Chat
      ocr_text = extract_text_with_ocr(file_data, api_key)
      response = call_chat_api(api_key, build_text_payload(ocr_text, direction))
    elsif content_type.starts_with?("image/")
      # Image: Vision
      response = call_chat_api(api_key, build_image_payload(file_data, content_type, direction))
    end

    parse_and_return(response, direction)
  end
end
```

### Извличане на текст от PDF

```crystal
private def self.extract_text_with_ocr(file_data : Bytes, api_key : String) : String
  # Кодиране като data URL
  base64_data = Base64.strict_encode(file_data)
  data_url = "data:application/pdf;base64,#{base64_data}"

  payload = {
    "model" => OCR_MODEL,
    "document" => {
      "type" => "document_url",
      "document_url" => data_url
    }
  }

  response = http_post(MISTRAL_OCR_URL, api_key, payload)
  result = JSON.parse(response)

  # Извличане на markdown от всички страници
  pages = result["pages"]?.try(&.as_a?) || []
  pages.map { |page| page["markdown"]?.try(&.as_s?) || "" }.join("\n\n")
end
```

### HTTP клиент

```crystal
private def self.call_chat_api(api_key : String, payload) : String
  uri = URI.parse(MISTRAL_CHAT_URL)
  client = HTTP::Client.new(uri.host.not_nil!, uri.port || 443, tls: true)
  client.read_timeout = 120.seconds  # Важно за големи документи

  headers = HTTP::Headers{
    "Authorization" => "Bearer #{api_key}",
    "Content-Type"  => "application/json",
  }

  response = client.post(uri.path.not_nil!, headers: headers, body: payload.to_json)

  if response.status_code != 200
    raise "Mistral API error: #{response.status_code} - #{response.body}"
  end

  response.body
end
```

---

## Формат на заявките

### Backend API endpoint

**POST** `/api/companies/:company_id/scanned_invoices/scan`

**Headers:**
```
Authorization: Bearer {JWT_TOKEN}
Content-Type: application/json
```

**Request Body:**
```json
{
  "file_content": "{BASE64_ENCODED_FILE}",
  "file_name": "invoice.pdf",
  "direction": "purchase",
  "vat_period": "2024-01"
}
```

| Поле | Тип | Описание |
|------|-----|----------|
| file_content | string | Base64 кодиран файл |
| file_name | string | Име на файла с разширение |
| direction | string | `purchase` или `sale` |
| vat_period | string | ДДС период (YYYY-MM) |

---

## Обработка на отговорите

### Очакван JSON от Mistral

```json
{
  "documentType": "INVOICE",
  "transactionType": "PURCHASE",
  "documentNumber": "0000001234",
  "documentDate": "2024-01-15",
  "dueDate": "2024-02-15",
  "counterpart": {
    "name": "Фирма ЕООД",
    "eik": "000000000",
    "vatNumber": "BG000000000",
    "address": "ул. Примерна 1, София"
  },
  "netAmount": 1000.00,
  "vatAmount": 200.00,
  "totalAmount": 1200.00
}
```

### Парсване и валидация

```crystal
private def self.parse_extraction(raw : String, direction : String) : ScannedInvoiceData
  # Почистване от markdown code blocks
  cleaned = clean_json_block(raw)
  json = JSON.parse(cleaned)

  invoice = ScannedInvoiceData.new(direction)
  invoice.invoice_number = json["documentNumber"]?.try(&.as_s?)
  invoice.invoice_date = parse_date(json["documentDate"]?.try(&.as_s?))
  # ... останалите полета

  # Извличане на контрагент
  if counterpart = json["counterpart"]?
    if direction == "purchase"
      invoice.vendor_name = counterpart["name"]?.try(&.as_s?)
      invoice.vendor_vat_number = normalize_vat(counterpart["vatNumber"]?.try(&.as_s?))
    else
      invoice.customer_name = counterpart["name"]?.try(&.as_s?)
      invoice.customer_vat_number = normalize_vat(counterpart["vatNumber"]?.try(&.as_s?))
    end
  end

  # Проверка за липсващи полета
  validate_and_set_confidence(invoice)

  invoice
end
```

### Нормализация на ДДС номер

```crystal
private def normalize_vat(vat : String?) : String?
  return nil if vat.nil? || vat.empty?

  cleaned = vat.strip.upcase
  cleaned = "BG#{cleaned}" unless cleaned.starts_with?("BG")
  cleaned
end
```

---

## Prompt Engineering

### Основен prompt

```text
Анализирай тази българска фактура и върни САМО валиден JSON.

⚠️ ПОТРЕБИТЕЛЯТ УКАЗА: Това е {PURCHASE/SALE}

Структура на JSON:
{
  "documentType": "INVOICE" | "CREDIT_NOTE" | "DEBIT_NOTE",
  "transactionType": "PURCHASE" | "SALE",
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
- За ПОКУПКА: counterpart е ДОСТАВЧИКЪТ (издателят на фактурата)
- За ПРОДАЖБА: counterpart е КЛИЕНТЪТ (получателят на фактурата)

Правила за идентификатори:
- vatNumber: ЗАДЪЛЖИТЕЛНО с префикс BG (напр. BG000000000)
- eik: САМО цифри, 9 или 13 знака, БЕЗ BG

Отговори САМО с JSON без пояснения.
```

### Защо е важен direction параметъра

При продажба и покупка контрагентът е различен:

| Тип | Контрагент | Секция във фактурата |
|-----|------------|---------------------|
| PURCHASE | Доставчик | "Издател", "Продавач", "Доставчик" |
| SALE | Клиент | "Получател", "Купувач", "Клиент" |

---

## Конфигурация

### Получаване на API ключ

1. Регистрирайте се на https://console.mistral.ai/
2. Създайте API ключ
3. Конфигурирайте в приложението:
   - **Вариант 1:** Настройки на фирмата → Интеграции → Mistral AI
   - **Вариант 2:** Environment variable `MISTRAL_API_KEY`

### Приоритет на конфигурацията

```crystal
def get_api_key(company : Company) : String
  # 1. Първо проверяваме настройките на фирмата
  if key = company.mistral_api_key
    return key unless key.empty?
  end

  # 2. После environment variable
  if key = ENV["MISTRAL_API_KEY"]?
    return key unless key.empty?
  end

  raise "Моля конфигурирайте Mistral API ключ"
end
```

### Съхранение в БД

API ключът се съхранява в JSON полето `settings` на компанията:

```json
{
  "mistral": {
    "enabled": true,
    "api_key": "sk-..."
  }
}
```

---

## Примерен код

### Frontend: Upload и изпращане

```rust
// Leptos/Rust
async fn upload_file(file: web_sys::File, direction: String, company_id: i64) {
    // Четене на файла като bytes
    let array_buffer = JsFuture::from(file.array_buffer()).await?;
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let bytes: Vec<u8> = uint8_array.to_vec();

    // Base64 кодиране
    let base64_content = base64::encode(&bytes);

    // Изпращане към API
    let payload = json!({
        "direction": direction,
        "vat_period": "2024-01",
        "file_content": base64_content,
        "file_name": file.name(),
    });

    let url = format!("/api/companies/{}/scanned_invoices/scan", company_id);
    let response = api_post(&url, &payload).await?;
}
```

### Backend: Пълен endpoint

```crystal
class Api::ScannedInvoices::Scan < ApiAction
  post "/api/companies/:company_id/scanned_invoices/scan" do
    company = CompanyQuery.find(company_id)

    # Вземане на API ключ
    api_key = company.mistral_api_key || ENV["MISTRAL_API_KEY"]?
    raise "No API key" if api_key.nil? || api_key.empty?

    # Декодиране на файла
    file_content = params.from_json["file_content"].as_s
    file_data = Base64.decode(file_content)
    file_name = params.from_json["file_name"].as_s
    direction = params.from_json["direction"].as_s

    # Сканиране с Mistral
    result = MistralDocumentService.scan_invoice(
      file_data,
      api_key,
      direction,
      file_name
    )

    # Записване в БД
    scanned_invoice = SaveScannedInvoice.create!(
      company_id: company.id,
      direction: result.invoice.direction,
      vendor_name: result.invoice.vendor_name,
      vendor_vat_number: result.invoice.vendor_vat_number,
      invoice_number: result.invoice.invoice_number,
      invoice_date: result.invoice.invoice_date,
      subtotal: result.invoice.subtotal,
      total_tax: result.invoice.total_tax,
      invoice_total: result.invoice.invoice_total,
      confidence: result.invoice.confidence,
      requires_manual_review: result.invoice.requires_manual_review,
      mistral_raw_json: result.raw_json  # Съхраняваме оригиналния отговор
    )

    json({success: true, data: scanned_invoice})
  end
end
```

---

## Troubleshooting

### Чести проблеми

| Проблем | Причина | Решение |
|---------|---------|---------|
| Timeout | Голям файл | Увеличете `read_timeout` |
| Празен отговор | Нечетлив документ | Проверете качеството на сканирането |
| Грешен контрагент | Объркан direction | Проверете дали direction е правилен |
| Invalid JSON | Markdown в отговора | Използвайте `clean_json_block()` |

### Логове

```crystal
Log.info { "Scanning invoice: #{file_name}" }
Log.info { "Direction: #{direction}" }
Log.info { "OCR text length: #{ocr_text.size}" }
Log.error { "Mistral error: #{ex.message}" }
```

---

## Версии и модели

| Компонент | Версия/Модел |
|-----------|--------------|
| OCR | `mistral-ocr-latest` |
| Vision | `pixtral-12b-2409` |
| Text analysis | `mistral-small-latest` |
| API версия | v1 |

---

## Лицензи и лимити

- Mistral API има rate limits и pricing
- Консултирайте https://docs.mistral.ai/ за актуална информация
- За production използвайте платен план

---

*Последна актуализация: Февруари 2026*
