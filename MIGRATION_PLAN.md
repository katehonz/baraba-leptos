# Интелигентна миграция от old/ към baraba-2

## Цел

Мигриране на функционалността от **KanKrum** (Java/Spring + React/TypeScript) към **Baraba-2** (Crystal/Lucky + Karax Nim) чрез интелигентен подход с подобрения.

---

## Архитектурно сравнение

| Аспект | KanKrum (old/) | Baraba-2 (new) | Подобрение |
|--------|----------------|----------------|-----------|
| Backend | Java 21 / Spring Boot 3.2 | Crystal 1.16+ / Lucky Framework | По-бърз компилация, по-малък footprint |
| Frontend | React 18 + TypeScript | Karax Nim | Статично типизиран, компилира се до JS |
| Database | PostgreSQL 16 | PostgreSQL 14+ | Същата база, JSONB оптимизация |
| Auth | JWT + Spring Security | JWT + Authentic | По-проста имплементация |
| API | REST + Swagger | REST + JSON API | По-чист дизайн |
| State | React Context | Nim vars + stores | По-ефективно управление |
| Styling | Material UI | Inline CSS + Tailwind (опция) | По-малък bundle |

---

## Миграционна стратегия

### Принципи
1. **Не копирай на 100%** - използвай learnings от old/
2. **Оптимизирай чрез Crystal/Nim** - използвай силните страни на новите технологии
3. **JSONB за сложни връзки** - journal_lines, invoice_lines в JSONB
4. **Модулна архитектура** - по-добра организация на кода
5. **Микросервизи само ако е нужно** - започни monolith, разделяй по-късно

---

## Фаза 1: Database Schema & Models

### Core Models (Priority 1)
Мигриране на основните модели с JSONB оптимизация:

```crystal
# backend/src/models/company.cr
class Company < BaseModel
  table do
    column name : String
    column eik : String
    column vat_number : String?
    column address : String?
    column city : String?
    column country : String?
    column post_code : String?
    column phone : String?
    column email : String?
    column website : String?
    column manager_name : String?
    column manager_eik : String?
    column accountant_name : String?
    column accountant_egn : String?
    column tax_authority : String?
    column inventory_valuation_method : String # "FIFO", "LIFO", "WEIGHTED_AVG"
    column is_vat_registered : Bool = false
    column nap_office : String?
    column vat_period : String = "monthly"
    column currency : String = "EUR"
    column fiscal_year_start_month : Int32 = 1

    # Settings (JSONB - по-гъвкаво от отделни колони)
    column settings : JSONB::Any = {} of String => JSON::Any

    # Timestamps
    column created_at : Time
    column updated_at : Time
  end
end
```

**ПОДОБРЕНИЕ:** Използваме JSONB за settings вместо 20+ отделни колони (azure_di_endpoint, saltedge_app_id, wise_api_key, и т.н.)

```crystal
# backend/src/models/journal_entry.cr
class JournalEntry < BaseModel
  table do
    column date : Time
    column description : String
    column status : String # "DRAFT", "POSTED"
    column document_number : String?
    column document_date : Time?
    column vat_purchase_operation : String?
    column vat_sales_operation : String?
    column total_amount : Float64?
    column total_vat_amount : Float64?
    column payment_method_code : String?
    column vat_period : String?

    # Lines в JSONB (по-бързо от JOIN)
    column lines : JSONB::Any = [] of JSON::Any

    # Foreign keys
    belongs_to company : Company
    belongs_to user : User

    timestamps
  end
end
```

**ПОДОБРЕНИЕ:** `lines` в JSONB вместо отделна таблица `journal_entry_lines`

### Reference Data Models (Priority 2)
```crystal
# backend/src/models/account.cr
class Account < BaseModel
  table do
    column code : String
    column name : String
    column account_type : String # "ASSET", "LIABILITY", "EQUITY", "REVENUE", "EXPENSE"
    column is_active : Bool = true
    column parent_account_id : Int64?

    belongs_to company : Company

    timestamps
  end
end

# backend/src/models/counterpart.cr
class Counterpart < BaseModel
  table do
    column name : String
    column counterpart_type : String # "CUSTOMER", "SUPPLIER", "BOTH"
    column vat_number : String?
    column eik : String?
    column address : String?
    column city : String?
    column country : String?
    column post_code : String?
    column contact_person : String?
    column email : String?
    column phone : String?

    belongs_to company : Company

    timestamps
  end
end
```

### Invoice Models (Priority 3)
```crystal
# backend/src/models/invoice.cr
class Invoice < BaseModel
  table do
    column invoice_number : String
    column document_type : String # "INVOICE", "DEBIT_NOTE", "CREDIT_NOTE"
    column original_invoice_id : Int64?
    column issue_date : Time
    column due_date : Time?
    column total_net_amount : Float64
    column total_vat_amount : Float64
    column total_amount : Float64
    column currency_code : String = "BGN"
    column exchange_rate : Float64 = 1.0
    column payment_method : String? # "BANK", "CASH"
    column notes : String?
    column status : String = "DRAFT"

    # VAT Compliance
    column tax_event_date : Time?
    column vat_exemption_reason : String?
    column has_inventory : Bool = false

    # Lines в JSONB
    column lines : JSONB::Any = [] of JSON::Any

    belongs_to company : Company
    belongs_to counterpart : Counterpart
    belongs_to journal_entry : JournalEntry?

    timestamps
  end
end
```

---

## Фаза 2: Frontend Architecture (Karax Nim)

### Структура на модулите

```
frontend/src/
├── app.nim                    # Основно приложение
├── frontend/
│   ├── api.nim                # API helpers (HTTP заявки)
│   ├── auth.nim               # Auth helpers (JWT управление)
│   ├── stores/                # State management
│   │   ├── company.nim
│   │   ├── invoice.nim
│   │   ├── journal_entry.nim
│   │   ├── account.nim
│   │   ├── counterpart.nim
│   │   └── dashboard.nim
│   ├── components/            # Reusable компоненти
│   │   ├── button.nim
│   │   ├── input.nim
│   │   ├── select.nim
│   │   ├── table.nim
│   │   ├── modal.nim
│   │   └── toast.nim
│   └── pages/                # Page компоненти
│       ├── login.nim
│       ├── dashboard.nim
│       ├── companies.nim
│       ├── invoices.nim
│       ├── journal_entries.nim
│       ├── accounts.nim
│       ├── counterparts.nim
│       └── settings.nim
└── types/                     # Type definitions
    ├── models.nim
    └── enums.nim
```

### API Module (frontend/src/frontend/api.nim)

```nim
import std/[asyncjs, json, dom, jsffi]

const API_BASE = "http://localhost:5000"

type
  RequestOptions = object
    method: string
    headers: seq[tuple[key, value: string]]
    body: string

proc apiGet(endpoint: string, token: string = ""): Future[JsonNode] {.async.} =
  let headers = @[
    ("Content-Type", "application/json"),
    if token != "": ("Authorization", "Bearer " & token) else: ("", "")
  ]

  let options = jsJson()
  jsSet(options, "method", "GET")

  let headersObj = jsJson()
  for (key, value) in headers:
    if value != "":
      jsSet(headersObj, cstring(key), cstring(value))
  jsSet(options, "headers", headersObj)

  let resp = await jsFetch(cstring(API_BASE & endpoint), options)
  let text = await jsText(resp)
  result = parseJson($text)

proc apiPost(endpoint: string, body: JsonNode, token: string = ""): Future[JsonNode] {.async.} =
  let headers = @[
    ("Content-Type", "application/json"),
    if token != "": ("Authorization", "Bearer " & token) else: ("", "")
  ]

  let options = jsJson()
  jsSet(options, "method", "POST")

  let headersObj = jsJson()
  for (key, value) in headers:
    if value != "":
      jsSet(headersObj, cstring(key), cstring(value))
  jsSet(options, "headers", headersObj)
  jsSet(options, "body", cstring($body))

  let resp = await jsFetch(cstring(API_BASE & endpoint), options)
  let text = await jsText(resp)
  result = parseJson($text)

proc apiPut(endpoint: string, body: JsonNode, token: string = ""): Future[JsonNode] {.async.}
proc apiDelete(endpoint: string, token: string = ""): Future[JsonNode] {.async.}
```

### Store Pattern (frontend/src/frontend/stores/invoice.nim)

```nim
import std/[asyncjs, json, sequtils]
import ../api

type
  Invoice = object
    id: int
    invoice_number: string
    issue_date: string
    total_amount: float64
    status: string

  InvoiceStore = ref object
    invoices: seq[Invoice]
    loading: bool
    error: string
    selectedInvoice: Invoice

var invoiceStore = InvoiceStore(
  invoices: @[],
  loading: false,
  error: ""
)

proc fetchInvoices(store: InvoiceStore, companyId: int, token: string): Future[void] {.async.} =
  store.loading = true
  try:
    let data = await apiGet("/api/companies/" & $companyId & "/invoices", token)
    store.invoices = data["invoices"].to(seq[Invoice])
  except:
    store.error = "Failed to fetch invoices"
  store.loading = false

proc createInvoice(store: InvoiceStore, companyId: int, invoice: JsonNode, token: string): Future[bool] {.async.} =
  try:
    let data = await apiPost("/api/companies/" & $companyId & "/invoices", invoice, token)
    await store.fetchInvoices(companyId, token)
    return true
  except:
    store.error = "Failed to create invoice"
    return false

proc updateInvoice(store: InvoiceStore, id: int, invoice: JsonNode, token: string): Future[bool] {.async.}
proc deleteInvoice(store: InvoiceStore, id: int, token: string): Future[bool] {.async.}
```

---

## Фаза 3: Priority-Based Implementation

### Priority 1: Core Accounting (Месец 1)
- [ ] Company CRUD
- [ ] Account CRUD (Chart of Accounts)
- [ ] Counterpart CRUD
- [ ] Journal Entry CRUD с JSONB lines
- [ ] Posting логика (DRAFT → POSTED)

### Priority 2: Invoicing (Месец 2)
- [ ] Invoice CRUD с JSONB lines
- [ ] Auto-постинг на фактури към Journal Entries
- [ ] ДДС калкулации
- [ ] Invoice Lines (products, VAT rates)

### Priority 3: Banking & Cash (Месец 3)
- [ ] Bank Accounts CRUD
- [ ] Bank Transactions CRUD
- [ ] Bank Reconciliation
- [ ] Cash Accounts

### Priority 4: VAT & Reports (Месец 4)
- [ ] VAT Returns
- [ ] Balance Sheet
- [ ] Income Statement
- [ ] Trial Balance
- [ ] SAFT-T Export

### Priority 5: Advanced Features (Месец 5-6)
- [x] Document Scanning (Mistral AI) ✅
- [ ] Salt Edge Integration
- [ ] Wise Integration
- [ ] VIES Validation
- [ ] Multi-currency
- [ ] Fixed Assets
- [ ] Stock/Inventory

---

## Фаза 4: Интелигентни подобрения

### 1. JSONB за сложни връзки
**Old:** Отделни таблици `journal_entry_lines`, `invoice_lines`
**New:** JSONB колони в основните таблици

**Предимства:**
- По-бързи queries (без JOIN)
- По-прост код
- По-лесно за backup/migrate
- PostgreSQL е оптимизиран за JSONB

### 2. Settings в JSONB
**Old:** 20+ отделни колони за всяка настройка
**New:** Една JSONB колона `settings`

```crystal
# company.settings:
{
  "mistral_ai": {
    "api_key": "...",
    "enabled": true
  },
  "saltedge": {
    "app_id": "...",
    "secret": "...",
    "enabled": false
  }
}
```

### 3. Статично типизиран frontend
**Old:** React + TypeScript (време за компилация: 10-30s)
**New:** Karax Nim (време за компилация: 1-3s)

**Предимства:**
- По-бърза разработка
- По-малък bundle (по-малко runtime)
- По-добра производителност

### 4. Модулна архитектура
**Old:** Monolithic frontend с глобални stores
**New:** Отделни Nim модули за всеки domain

### 5. По-прост auth
**Old:** Spring Security с много конфигурации
**New:** Lucky Authentic с JWT

---

## Фаза 5: Микросервизи (Само ако е нужно)

### Архитектура за микросервизи

```
┌─────────────────────────────────────┐
│         Frontend (Karax Nim)        │
│            Port 3000                │
└─────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────┐
│         API Gateway / Load Balancer  │
│              Nginx / Traefik          │
└─────────────────────────────────────┘
                 │
        ┌────────┼────────┐
        ▼        ▼        ▼
┌──────────┐ ┌──────────┐ ┌──────────┐
│ Core API │ │ Scanner  │ │ Reports  │
│ (Lucky)  │ │ (Nim?)   │ │ (Jasper) │
│ Port 5000│ │ Port 5001│ │ Port 5005│
└──────────┘ └──────────┘ └──────────┘
        │
        ▼
┌────────────────────┐
│   PostgreSQL      │
│   Port 5432       │
└────────────────────┘
```

### Кога да използваме микросервизи?
- **Да:** Ако отделните компоненти имат различни scalability изисквания
- **Да:** Ако искаме различни езици за различни компоненти (Scanner на Nim?)
- **Не:** В началото - започни с monolith, раздели по-късно

---

## Фаза 6: Data Migration Plan

### От old/ към baraba-2

```sql
-- 1. Copy companies
INSERT INTO baraba2.companies (name, eik, vat_number, ...)
SELECT name, eik, vat_number, ...
FROM kankrum.companies;

-- 2. Copy accounts
INSERT INTO baraba2.accounts (...)
SELECT ... FROM kankrum.accounts;

-- 3. Migrate journal entries with lines to JSONB
INSERT INTO baraba2.journal_entries (
  date, description, status, company_id, lines
)
SELECT
  date,
  description,
  status,
  company_id,
  COALESCE(
    jsonb_agg(
      jsonb_build_object(
        'account_id', account_id,
        'debit', debit,
        'credit', credit,
        'description', description
      )
    ),
    '[]'::jsonb
  ) as lines
FROM kankrum.journal_entries
LEFT JOIN kankrum.journal_entry_lines ON journal_entry_lines.journal_entry_id = journal_entries.id
GROUP BY journal_entries.id;
```

---

## Фаза 7: Testing Strategy

### Backend Tests (Crystal)
```crystal
# spec/actions/api/companies_spec.cr
describe Api::Companies::Index do
  it "returns all companies for authenticated user" do
    user = UserBox.create
    company = CompanyBox.create(user_id: user.id)

    response = ApiClient.auth(user).get("/api/companies")

    response.status_code.should eq 200
    response.json["companies"].size.should eq 1
  end
end
```

### Frontend Tests (Nim)
```nim
import unittest

test "InvoiceStore fetchInvoices":
  let store = InvoiceStore()
  await store.fetchInvoices(1, "fake-token")
  check(store.loading == false)
```

---

## Фаза 8: Deployment

### Docker Compose (Development)
```yaml
version: '3.8'
services:
  db:
    image: postgres:16
    environment:
      POSTGRES_DB: lucky
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: your_password_here
    ports:
      - "5432:5432"

  backend:
    build: ./backend
    ports:
      - "5000:5000"
    depends_on:
      - db

  frontend:
    build: ./frontend
    ports:
      - "3000:80"
    depends_on:
      - backend
```

---

## Следващи стъпки (Immediate)

### Week 1-2: Setup
1. ✅ Update documentation (completed)
2. ⏳ Create database migrations
3. ⏳ Implement Company model & API
4. ⏳ Implement Company CRUD in frontend

### Week 3-4: Core Models
1. ⏳ Implement Account model & API
2. ⏳ Implement Counterpart model & API
3. ⏳ Implement Journal Entry model & API (JSONB)
4. ⏳ Implement frontend stores

### Week 5-6: Invoicing
1. ⏳ Implement Invoice model & API (JSONB)
2. ⏳ Implement Invoice CRUD in frontend
3. ⏳ Implement auto-posting logic
4. ⏳ VAT calculations

---

## Ключови преимущества на новата архитектура

1. **По-бързо компилиране:** Crystal (1-3s) vs Java (10-30s)
2. **По-малък footprint:** Nim → JavaScript bundle е по-малък
3. **По-проста база:** JSONB вместо много JOINs
4. **По-добра производителност:** Статично типизиран frontend
5. **По-лесно maintain:** По-малко код, по-чиста архитектура

---

## Заключение

Миграцията не е просто копиране на код, а:
- Използване на learnings от стария проект
- Оптимизация чрез новите технологии (Crystal, Nim, JSONB)
- Подобрена архитектура с по-добра scalability
- По-проста и по-бърза разработка

**Статус:** Планът е готов за изпълнение! 🚀
