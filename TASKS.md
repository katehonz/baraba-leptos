# Baraba - Задачи за разработка

## Текущо състояние

Frontend (Karax Nim) е готов с основните страници:
- ✅ Login
- ✅ Dashboard
- ✅ Invoices (placeholder)
- ✅ Accounts (placeholder)
- ✅ Counterparts (placeholder)
- ✅ Products (placeholder)
- ✅ Settings (placeholder)

Backend (Lucky Framework) е базова структура.

---

## Приоритетни задачи

### 1. Backend API - Автентикация
**Файлове:** `backend/src/actions/api/`

- [x] Имплементирай `/api/sign_ins` endpoint за реален login
- [x] Имплементирай `/api/sign_ups` endpoint за регистрация
- [x] Добави JWT token генериране и валидация
- [x] Свържи с PostgreSQL база `lucky`

```crystal
# Пример за sign_in action
class Api::SignIns::Create < ApiAction
  post "/api/sign_ins" do
    # Валидирай username/password
    # Генерирай JWT token
    # Върни { token: "...", username: "..." }
  end
end
```

---

### 2. Backend API - CRUD операции

#### Companies (Фирми)
- [ ] `GET /api/companies` - списък фирми
- [ ] `POST /api/companies` - създаване
- [ ] `GET /api/companies/:id` - детайли
- [ ] `PUT /api/companies/:id` - редакция
- [ ] `DELETE /api/companies/:id` - изтриване

#### Journal Entries (Счетоводни записи)
- [ ] `GET /api/companies/:id/entries` - списък записи
- [ ] `POST /api/companies/:id/entries` - нов запис
- [ ] `PUT /api/entries/:id` - редакция
- [ ] `DELETE /api/entries/:id` - изтриване
- [ ] `POST /api/entries/:id/post` - осчетоводяване

#### Invoices (Фактури)
- [ ] `GET /api/companies/:id/invoices` - списък
- [ ] `POST /api/companies/:id/invoices` - създаване
- [ ] `GET /api/invoices/:id/pdf` - PDF генериране

#### Accounts (Сметки)
- [ ] `GET /api/companies/:id/accounts` - сметкоплан
- [ ] `POST /api/companies/:id/accounts` - нова сметка

#### Counterparts (Контрагенти)
- [ ] `GET /api/companies/:id/counterparts` - списък
- [ ] `POST /api/companies/:id/counterparts` - създаване
- [ ] Интеграция с VIES за проверка на ДДС номер

---

### 3. Frontend - HTTP и Auth модули

**Нов файл:** `frontend/src/frontend/api.nim`

```nim
import std/[asyncjs, json, dom, jsffi]

proc apiGet(endpoint: string): Future[JsonNode] {.async.} =
  let response = await fetch(API_BASE & endpoint)
  let text = await response.text()
  result = parseJson($text)

proc apiPost(endpoint: string, body: JsonNode): Future[JsonNode] {.async.} =
  let headers = jsJson()
  jsSet(headers, "Content-Type", "application/json")
  if state.token != "":
    jsSet(headers, "Authorization", cstring("Bearer " & state.token))

  let options = jsJson()
  jsSet(options, "method", "POST")
  jsSet(options, "headers", headers)
  jsSet(options, "body", cstring($body))

  let resp = await jsFetch(cstring(API_BASE & endpoint), options)
  let text = await jsText(resp)
  result = parseJson($text)
```

**Нов файл:** `frontend/src/frontend/auth.nim`

```nim
type
  AuthStore = ref object
    token: string
    email: string
    isAuthenticated: bool

proc login(email: string, password: string): Future[bool] {.async.} =
  let body = %*{"user": {"email": email, "password": password}}
  let resp = await apiPost("/api/sign_ins", body)
  if resp{"success"}.getBool(false):
    state.token = resp{"token"}.getStr("")
    state.email = email
    state.isAuthenticated = true
    window.localStorage.setItem("token", state.token)
    window.localStorage.setItem("email", state.email)
    return true
  return false

proc logout(): void =
  state.token = ""
  state.email = ""
  state.isAuthenticated = false
  window.localStorage.removeItem("token")
  window.localStorage.removeItem("email")
```

---

### 4. Frontend - Динамични данни

#### Dashboard
- [ ] Зареждай статистики от API
- [ ] Показвай реални числа за entries, invoices, revenue

#### Journal Entries
- [ ] Зареждай записи от `/api/companies/:id/entries`
- [ ] Форма за нов запис с debit/credit линии
- [ ] Бутон за осчетоводяване

#### Invoices
- [ ] Зареждай фактури от API
- [ ] Форма за нова фактура с линии
- [ ] PDF download бутон

---

### 5. Database миграции

**Директория:** `backend/db/migrations/`

```crystal
# 00000000000002_create_companies.cr
class CreateCompanies::V00000000000002 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(Company) do
      primary_key id : Int64
      add name : String
      add vat_number : String?
      add address : String?
      add_timestamps
    end
  end
end

# 00000000000003_create_journal_entries.cr
# 00000000000004_create_accounts.cr
# 00000000000005_create_invoices.cr
# 00000000000006_create_counterparts.cr
```

---

### 6. Stores за управление на състоянието

**Файлове:** `frontend/src/frontend/stores/`

- [ ] `company.nim` - CompanyStore за CRUD операции
- [ ] `invoices.nim` - InvoicesStore
- [ ] `accounts.nim` - AccountsStore
- [ ] `counterparts.nim` - CounterpartsStore
- [ ] `dashboard.nim` - DashboardStore за статистики

---

## Допълнителни задачи

### Reports
- [ ] Balance Sheet report
- [ ] Income Statement report
- [ ] Trial Balance report
- [ ] PDF/Excel export

### VAT
- [ ] VAT return калкулация
- [ ] VIES интеграция за проверка на EU VAT номера
- [ ] XML export за НАП

### Settings
- [ ] Company settings форма
- [ ] User profile редакция
- [ ] Password change

---

## Тестване

```bash
# Backend тестове
cd backend
crystal spec

# Frontend тестове
cd frontend
nimble test
```

---

## Стартиране

```bash
# Всичко заедно
./start_local.sh

# Само frontend
cd frontend
nim js src/app.nim
python3 -m http.server 3000

# Само backend
cd backend
lucky dev

# Спиране
./stop_local.sh
```

---

## Връзки

- **Frontend:** http://localhost:3000
- **Backend:** http://localhost:5000
- **Database:** PostgreSQL `lucky` на localhost:5432

---

## Бележки

- Паролата за PostgreSQL е `your_password_here`
- JWT secret е в `backend/.env`
- Nim компилира се до JavaScript (`nim js src/app.nim`)
- Frontend се сервира с HTTP server (python3 или npx serve)
- Karax е виртуален DOM framework за Nim, подобен на React

---

**Последна актуализация:** 2026-01-26
