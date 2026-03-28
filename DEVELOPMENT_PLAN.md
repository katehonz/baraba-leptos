# План за продължаване на разработката - Baraba-2

## Дата: 2026-01-26
## Статус: В процес
## Архитектура: Crystal/Lucky Backend + Karax Nim Frontend

---

## Ситуация

### Текущ проект (baraba-2)
- **Backend**: Crystal/Lucky Framework - базова структура
- **Frontend**: Karax Nim (компилира се до JavaScript)
- **Database**: PostgreSQL (lucky)

### Референционен проект (kankrum)
- **Backend**: Java Spring - пълнофункционален
- **Frontend**: React - пълнофункционален
- **Пълна система за счетоводство**

---

## Текущо състояние на baraba-2

### Backend (Crystal/Lucky)
✅ Завършено:
- User model (email, encrypted_password)
- Auth actions: /api/sign_ins, /api/sign_ups, /api/me
- UserToken model
- JWT token генериране
- Базова база данни с users таблица

⏳ Предстои:
- Company модел и API
- Journal Entry модел и API
- Invoice модел и API
- Account модел и API
- Counterpart модел и API
- Database миграции за всички таблици

### Frontend (Karax Nim)
✅ Завършено:
- Основно приложение с Karax framework
- App state management
- Login страница с форма
- Страници: Dashboard, Invoices, Accounts, Counterparts, Products, Settings
- Sidebar navigation
- Basic CSS стилове

⏳ Предстои:
- HTTP модул за API заявки
- Auth модул за управление на токени
- Реален login с API
- Динамични данни от API
- CRUD форми
- Валидация и error handling

---

## План за разработка

### Фаза 1: Database миграции и Models

#### 1.1 Company модел
```
backend/src/models/company.cr
- id: Int64 (PK)
- name: String
- vat_number: String?
- address: String?
- created_at: Time
- updated_at: Time
```

**Миграция**: `00000000000002_create_companies.cr`

#### 1.2 Account модел
```
backend/src/models/account.cr
- id: Int64 (PK)
- company_id: Int64 (FK)
- code: String
- name: String
- account_type: String (asset, liability, equity, revenue, expense)
- is_active: Bool
- created_at: Time
- updated_at: Time
```

**Миграция**: `00000000000003_create_accounts.cr`

#### 1.3 Counterpart модел
```
backend/src/models/counterpart.cr
- id: Int64 (PK)
- company_id: Int64 (FK)
- name: String
- vat_number: String?
- address: String?
- contact_person: String?
- email: String?
- phone: String?
- counterpart_type: String (customer, vendor, both)
- created_at: Time
- updated_at: Time
```

**Миграция**: `00000000000004_create_counterparts.cr`

#### 1.4 Journal Entry модел
```
backend/src/models/journal_entry.cr
- id: Int64 (PK)
- company_id: Int64 (FK)
- entry_date: Time
- description: String
- reference: String?
- status: String (draft, posted)
- created_at: Time
- updated_at: Time
```

**Миграция**: `00000000000005_create_journal_entries.cr`

#### 1.5 Journal Line модел
```
backend/src/models/journal_line.cr
- id: Int64 (PK)
- journal_entry_id: Int64 (FK)
- account_id: Int64 (FK)
- debit: Float64?
- credit: Float64?
- description: String?
- created_at: Time
- updated_at: Time
```

**Миграция**: `00000000000006_create_journal_lines.cr`

#### 1.6 Invoice модел
```
backend/src/models/invoice.cr
- id: Int64 (PK)
- company_id: Int64 (FK)
- counterpart_id: Int64 (FK)
- invoice_number: String
- invoice_date: Time
- due_date: Time?
- subtotal: Float64
- vat_amount: Float64
- total_amount: Float64
- currency: String (EUR, BGN, USD)
- status: String (draft, sent, paid, overdue)
- notes: String?
- created_at: Time
- updated_at: Time
```

**Миграция**: `00000000000007_create_invoices.cr`

#### 1.7 Invoice Line модел
```
backend/src/models/invoice_line.cr
- id: Int64 (PK)
- invoice_id: Int64 (FK)
- description: String
- quantity: Float64
- unit_price: Float64
- vat_rate: Float64
- total: Float64
- account_id: Int64 (FK)
- created_at: Time
- updated_at: Time
```

**Миграция**: `00000000000008_create_invoice_lines.cr`

---

### Фаза 2: Backend API - CRUD операции

#### 2.1 Companies API
```
GET    /api/companies              - списък фирми
POST   /api/companies              - създаване
GET    /api/companies/:id          - детайли
PUT    /api/companies/:id          - редакция
DELETE /api/companies/:id          - изтриване
```

**Файлове**:
- `backend/src/actions/api/companies/index.cr`
- `backend/src/actions/api/companies/create.cr`
- `backend/src/actions/api/companies/show.cr`
- `backend/src/actions/api/companies/update.cr`
- `backend/src/actions/api/companies/delete.cr`

#### 2.2 Accounts API
```
GET    /api/companies/:id/accounts  - сметкоплан
POST   /api/companies/:id/accounts  - нова сметка
PUT    /api/accounts/:id            - редакция
DELETE /api/accounts/:id            - изтриване
```

**Файлове**:
- `backend/src/actions/api/accounts/index.cr`
- `backend/src/actions/api/accounts/create.cr`
- `backend/src/actions/api/accounts/update.cr`
- `backend/src/actions/api/accounts/delete.cr`

#### 2.3 Counterparts API
```
GET    /api/companies/:id/counterparts  - списък
POST   /api/companies/:id/counterparts  - създаване
PUT    /api/counterparts/:id            - редакция
DELETE /api/counterparts/:id            - изтриване
```

**Файлове**:
- `backend/src/actions/api/counterparts/index.cr`
- `backend/src/actions/api/counterparts/create.cr`
- `backend/src/actions/api/counterparts/update.cr`
- `backend/src/actions/api/counterparts/delete.cr`

#### 2.4 Journal Entries API
```
GET    /api/companies/:id/entries    - списък записи
POST   /api/companies/:id/entries    - нов запис
GET    /api/entries/:id              - детайли
PUT    /api/entries/:id              - редакция
DELETE /api/entries/:id              - изтриване
POST   /api/entries/:id/post         - осчетоводяване
```

**Файлове**:
- `backend/src/actions/api/entries/index.cr`
- `backend/src/actions/api/entries/create.cr`
- `backend/src/actions/api/entries/show.cr`
- `backend/src/actions/api/entries/update.cr`
- `backend/src/actions/api/entries/delete.cr`
- `backend/src/actions/api/entries/post.cr`

#### 2.5 Invoices API
```
GET    /api/companies/:id/invoices    - списък
POST   /api/companies/:id/invoices    - създаване
GET    /api/invoices/:id              - детайли
PUT    /api/invoices/:id              - редакция
DELETE /api/invoices/:id              - изтриване
GET    /api/invoices/:id/pdf          - PDF генериране
```

**Файлове**:
- `backend/src/actions/api/invoices/index.cr`
- `backend/src/actions/api/invoices/create.cr`
- `backend/src/actions/api/invoices/show.cr`
- `backend/src/actions/api/invoices/update.cr`
- `backend/src/actions/api/invoices/delete.cr`
- `backend/src/actions/api/invoices/pdf.cr`

---

### Фаза 3: Frontend - HTTP и Auth модули

#### 3.1 HTTP модул (API помощник)
```nim
# frontend/src/frontend/api.nim
import std/[asyncjs, json, dom, jsffi]

type
  RequestOptions = object
    method: string
    headers: seq[tuple[key, value: string]]
    body: string

proc apiRequest(url: string, options: RequestOptions): Future[JsonNode] {.async.} =
  # Implementation
  pass

proc apiGet(endpoint: string): Future[JsonNode] {.async.}
proc apiPost(endpoint: string, body: JsonNode): Future[JsonNode] {.async.}
proc apiPut(endpoint: string, body: JsonNode): Future[JsonNode] {.async.}
proc apiDelete(endpoint: string): Future[JsonNode] {.async.}
```

**Файл**: `frontend/src/frontend/api.nim`

#### 3.2 Auth модул
```nim
# frontend/src/frontend/auth.nim
import std/[asyncjs, json, dom, jsffi]

type
  AuthStore = ref object
    token: string
    email: string
    isAuthenticated: bool

proc login(email: string, password: string): Future[bool] {.async.}
proc logout(): void
proc checkAuth(): bool
```

**Файл**: `frontend/src/frontend/auth.nim`

#### 3.3 Актуализация на app.nim
- Интегриране на AuthStore
- Реален login с POST /api/sign_ins
- Token запазване в localStorage
- Error handling

**Файл**: `frontend/src/app.nim`

---

### Фаза 4: Frontend - Stores за управление на състоянието

#### 4.1 CompanyStore
```nim
# frontend/src/frontend/stores/company.nim
type
  Company = object
    id: int
    name: string
    vat_number: string

  CompanyStore = ref object
    companies: seq[Company]
    selectedCompany: Company
    loading: bool
    error: string

proc loadCompanies(): Future[void] {.async.}
proc createCompany(name: string, vatNumber: string): Future[bool] {.async.}
proc selectCompany(id: int): void
```

**Файл**: `frontend/src/frontend/stores/company.nim`

#### 4.2 DashboardStore
```nim
# frontend/src/frontend/stores/dashboard.nim
type
  DashboardStats = object
    totalInvoices: int
    totalRevenue: float64
    totalExpenses: float64
    totalCounterparts: int

  DashboardStore = ref object
    stats: DashboardStats
    loading: bool
    error: string

proc loadStats(): Future[void] {.async.}
```

**Файл**: `frontend/src/frontend/stores/dashboard.nim`

#### 4.3 InvoicesStore
```nim
# frontend/src/frontend/stores/invoices.nim
type
  Invoice = object
    id: int
    invoice_number: string
    invoice_date: string
    counterpart_name: string
    total_amount: float64
    status: string

  InvoicesStore = ref object
    invoices: seq[Invoice]
    loading: bool
    error: string

proc loadInvoices(companyId: int): Future[void] {.async.}
proc createInvoice(data: JsonNode): Future[bool] {.async.}
proc updateInvoice(id: int, data: JsonNode): Future[bool] {.async.}
proc deleteInvoice(id: int): Future[bool] {.async.}
```

**Файл**: `frontend/src/frontend/stores/invoices.nim`

---

### Фаза 5: Frontend - Динамични данни

#### 5.1 Dashboard
- Зареждане на статистики от API
- Реални числа вместо placeholder

**Файл**: `frontend/src/app.nim` (renderDashboard proc)

#### 5.2 Journal Entries
- Списък с записи от API
- Форма за нов запис с debit/credit линии
- Бутон за осчетоводяване

#### 5.3 Invoices
- Списък с фактури от API
- Форма за нова фактура с линии
- PDF download бутон

#### 5.4 Accounts
- Сметкоплан от API
- Форма за нова сметка

#### 5.5 Counterparts
- Списък с контрагенти от API
- Форма за нов контрагент
- VIES проверка за ДДС номер

---

### Фаза 6: Допълнителни функционалности

#### 6.1 Reports
- Balance Sheet
- Income Statement
- Trial Balance
- PDF/Excel export

#### 6.2 VAT
- VAT return калкулация
- VIES интеграция
- XML export за НАП

#### 6.3 Settings
- Company settings
- User profile
- Password change

---

## Ред на изпълнение

### Етап 1 (Backend - Database & Models)
1. ✅ User model (съществува)
2. ⏳ Company model + миграция
3. ⏳ Account model + миграция
4. ⏳ Counterpart model + миграция
5. ⏳ JournalEntry + JournalLine models + миграции
6. ⏳ Invoice + InvoiceLine models + миграции

### Етап 2 (Backend - API)
1. ✅ Auth API (съществува)
2. ⏳ Companies API
3. ⏳ Accounts API
4. ⏳ Counterparts API
5. ⏳ Journal Entries API
6. ⏳ Invoices API

### Етап 3 (Frontend - Infrastructure)
1. ✅ Basic UI structure (съществува)
2. ⏳ HTTP модул
3. ⏳ Auth модул
4. ⏳ CompanyStore
5. ⏳ DashboardStore
6. ⏳ InvoicesStore

### Етап 4 (Frontend - Pages with API)
1. ✅ Placeholder страници (съществуват)
2. ⏳ Login с реален API
3. ⏳ Dashboard с реални данни
4. ⏳ Journal Entries с CRUD
5. ⏳ Invoices с CRUD
6. ⏳ Accounts с CRUD
7. ⏳ Counterparts с CRUD

### Етап 5 (Допълнително)
1. ⏳ Reports
2. ⏳ VAT функционалности
3. ⏳ Settings

---

## Примерен код (Референция от kankrum)

### Java Spring - CompanyController (kankrum)
```java
@RestController
@RequestMapping("/api/companies")
public class CompanyController {
    @Autowired
    private CompanyService companyService;

    @GetMapping
    public List<Company> getAllCompanies() {
        return companyService.findAll();
    }

    @PostMapping
    public Company createCompany(@RequestBody Company company) {
        return companyService.save(company);
    }

    @GetMapping("/{id}")
    public Company getCompanyById(@PathVariable Long id) {
        return companyService.findById(id);
    }

    @PutMapping("/{id}")
    public Company updateCompany(@PathVariable Long id, @RequestBody Company company) {
        return companyService.update(id, company);
    }

    @DeleteMapping("/{id}")
    public void deleteCompany(@PathVariable Long id) {
        companyService.delete(id);
    }
}
```

### Crystal/Lucky - Companies::Index (baraba-2)
```crystal
class Api::Companies::Index < ApiAction
  get "/api/companies" do
    companies = CompanyQuery.new
    json(CompanySerializer.for_collection(companies))
  end
end
```

### Karax Nim - HTTP Request (baraba-2)
```nim
import std/[asyncjs, json, dom, jsffi]

proc apiPost(endpoint: string, body: JsonNode): Future[JsonNode] {.async.} =
  let options = jsJson()
  jsSet(options, "method", "POST")
  jsSet(options, "headers", jsHeaders())
  jsSet(options, "body", cstring($body))

  let resp = await jsFetch(cstring(API_BASE & endpoint), options)
  let text = await jsText(resp)
  result = parseJson($text)
```

---

## Следващи стъпки

### Непосредствено
1. ⏳ Създаване на Company model и миграция
2. ⏳ Тестване на миграцията
3. ⏳ Създаване на Companies API actions
4. ⏳ HTTP модул във frontend

### Краткосрочно
1. Завършване на всички models и миграции
2. Пълна CRUD функционалност за всички ресурси
3. Реален login във frontend

### Дългосрочно
1. Reports функционалности
2. VAT интеграция
3. PDF generation за фактури

---

## Бележки

- Всички промени са в baraba-2
- kankrum се ползва само за референция
- Crystal/Lucky следва същите принципи като Java Spring
- Karax Nim използва виртуален DOM и компоненти като React
- Nim се компилира до JavaScript за frontend

---

**Статус**: Планът е готов за изпълнение! 🚀
