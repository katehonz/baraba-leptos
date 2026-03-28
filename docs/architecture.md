# Архитектура на Baraba

## Преглед на системата

Baraba е full-stack счетоводно приложение, разработено с модерен стек технологии:

- **Backend:** Crystal 1.16+ с Lucky Framework 1.3
- **Frontend:** Rust с Leptos и WebAssembly
- **Database:** PostgreSQL 14+
- **Architecture:** REST API + SPA (Single Page Application)
- **OCR/AI:** Mistral AI

## Архитектурна схема

```
┌─────────────────┐
│   Browser       │
│  (Leptos WASM)  │
└────────┬────────┘
          │ HTTP/JSON
          │
          ▼
┌─────────────────┐
│  Lucky API      │
│  (Crystal)      │
│  - Actions      │
│  - Operations   │
│  - Services     │
└────────┬────────┘
          │ Avram ORM
          │
    ┌─────┴─────┐
    │           │
    ▼           ▼
┌─────────┐  ┌──────────────┐
│PostgreSQL│  │External APIs │
│ Database │  │ - ECB        │
│          │  │ - VIES       │
└─────────┘  │ - Mistral    │
              │ - SMTP       │
              └──────────────┘
```

## Директорийна структура

```
baraba-2/
├── backend/                   # Crystal / Lucky Framework
│   ├── config/               # Конфигурационни файлове
│   ├── db/                   # Миграции на базата данни
│   ├── src/
│   │   ├── actions/          # API endpoints и контролери
│   │   │   └── api/          # REST API actions
│   │   │       ├── accounting_periods/
│   │   │       ├── accounts/
│   │   │       ├── admin/
│   │   │       ├── auth/
│   │   │       ├── backup/
│   │   │       ├── bank_accounts/
│   │   │       ├── bank_transactions/
│   │   │       ├── companies/
│   │   │       ├── controlisy/
│   │   │       ├── counterparts/
│   │   │       ├── currencies/
│   │   │       ├── dividends/
│   │   │       ├── documents/
│   │   │       ├── exchange_rates/
│   │   │       ├── fixed_assets/
│   │   │       ├── invoices/
│   │   │       ├── journal_entries/
│   │   │       ├── nomenclatures/
│   │   │       ├── opening_balances/
│   │   │       ├── products/
│   │   │       ├── reports/
│   │   │       ├── roles/
│   │   │       ├── saft/
│   │   │       ├── scanned_invoices/
│   │   │       └── users/
│   │   ├── models/           # Avram модели (40+ модела)
│   │   │   ├── user.cr
│   │   │   ├── company.cr
│   │   │   ├── account.cr
│   │   │   ├── counterpart.cr
│   │   │   ├── invoice.cr
│   │   │   ├── invoice_line.cr
│   │   │   ├── payment.cr
│   │   │   ├── payment_line.cr
│   │   │   ├── journal_entry.cr
│   │   │   ├── journal_line.cr
│   │   │   ├── product.cr
│   │   │   ├── fixed_asset.cr
│   │   │   ├── fixed_asset_transaction.cr
│   │   │   ├── stock_transaction.cr
│   │   │   ├── physical_stock.cr
│   │   │   ├── warehouse.cr
│   │   │   ├── currency.cr
│   │   │   ├── exchange_rate.cr
│   │   │   ├── vat_return.cr
│   │   │   ├── vat_journal_entry.cr
│   │   │   ├── bank_account.cr
│   │   │   ├── bank_transaction.cr
│   │   │   ├── dividend.cr
│   │   │   ├── shareholder.cr
│   │   │   ├── document.cr
│   │   │   ├── scanned_invoice.cr
│   │   │   ├── accounting_period.cr
│   │   │   ├── opening_balance.cr
│   │   │   ├── user_company_role.cr
│   │   │   ├── role.cr
│   │   │   ├── permission.cr
│   │   │   ├── system_setting.cr
│   │   │   ├── controlisy_import.cr
│   │   │   ├── combined_nomenclature.cr
│   │   │   ├── iso_country.cr
│   │   │   ├── iso_currency.cr
│   │   │   ├── fixed_asset_category.cr
│   │   │   └── saft_*.cr (SAF-T номенклатури)
│   │   ├── operations/      # Business logic
│   │   ├── queries/         # Database queries
│   │   ├── services/        # Външни услуги
│   │   │   ├── saft_exporter.cr
│   │   │   ├── saft_asset_mapper.cr
│   │   │   ├── saft_movement_mapper.cr
│   │   │   ├── controlisy_importer.cr
│   │   │   ├── controlisy_parser.cr
│   │   │   ├── ecb_exchange_rate_service.cr
│   │   │   ├── vies_service.cr
│   │   │   ├── azure_document_service.cr
│   │   │   ├── mistral_document_service.cr
│   │   │   ├── email_service.cr
│   │   │   ├── database_backup.cr
│   │   │   ├── company_accounts_initializer.cr
│   │   │   ├── period_service.cr
│   │   │   └── access_control_service.cr
│   │   ├── handlers/        # Custom middleware
│   │   └── serializers/     # JSON serialization
│   ├── lib/                  # Lucky framework shards
│   └── spec/                 # Тестове
│
├── leptos/                   # Rust / Leptos Frontend
│   ├── src/
│   │   ├── main.rs          # Entry point
│   │   ├── app.rs           # App компонент
│   │   ├── router.rs        # Routing конфигурация
│   │   ├── api.rs           # API клиент
│   │   ├── models.rs        # Frontend модели
│   │   ├── context/         # App context
│   │   ├── constants/       # Константи
│   │   ├── components/      # Reusable UI компоненти
│   │   ├── pages/           # Страници/Views (30+ страници)
│   │   │   ├── login.rs
│   │   │   ├── register.rs
│   │   │   ├── forgot_password.rs
│   │   │   ├── reset_password.rs
│   │   │   ├── verify_email.rs
│   │   │   ├── dashboard.rs
│   │   │   ├── invoices.rs
│   │   │   ├── accounts.rs
│   │   │   ├── counterparts.rs
│   │   │   ├── products.rs
│   │   │   ├── fixed_assets.rs
│   │   │   ├── journal_entries.rs
│   │   │   ├── payments.rs
│   │   │   ├── vat_returns.rs
│   │   │   ├── currencies.rs
│   │   │   ├── exchange_rates.rs
│   │   │   ├── bank_accounts.rs
│   │   │   ├── bank_transactions.rs
│   │   │   ├── accounting_periods.rs
│   │   │   ├── opening_balances.rs
│   │   │   ├── warehouse.rs
│   │   │   ├── stock_transactions.rs
│   │   │   ├── documents.rs
│   │   │   ├── scanned_invoices.rs
│   │   │   ├── saft_export.rs
│   │   │   ├── controlisy_import.rs
│   │   │   ├── saft_movement_mappings.rs
│   │   │   ├── reports.rs
│   │   │   ├── settings.rs
│   │   │   ├── profile.rs
│   │   │   ├── users.rs
│   │   │   ├── roles.rs
│   │   │   ├── admin_dashboard.rs
│   │   │   └── system_settings.rs
│   │   ├── stores/          # State management
│   │   └── i18n/            # Преводи (BG/EN)
│   ├── assets/              # CSS, картинки
│   └── style/               # Глобални стилове
│
├── docs/                    # Документация
├── old/                     # Стара версия (референция)
└── *.sh                     # Скриптове за управление
```

## Backend архитектура

### Lucky Framework MVC Pattern

```
Request → Route → Action → Operation → Model → Database
                ↓
            Query → Database
                ↓
            Response (JSON)
```

### Директория `backend/src/actions/api/`

Actions са контролерите, които обработват HTTP заявки:

#### Auth & User Management
- `auth/` - Аутентикация (signin, signup, me, email verification, password reset)
- `users/` - Потребители CRUD
- `roles/` - Роли и права

#### Core Accounting
- `accounts/` - Сметкоплан CRUD
- `counterparts/` - Контрагенти (клиенти/доставчици)
- `companies/` - Фирми CRUD
- `journal_entries/` - Счетоводни записи
- `invoices/` - Фактури
- `payments/` - Плащания

#### Assets & Inventory
- `fixed_assets/` - Дълготрайни активи
- `products/` - Продукти и услуги
- `warehouse/` - Складове (ако има отделна папка)
- `documents/` - Документи

#### Financial & Reporting
- `bank_accounts/` - Банкови сметки
- `bank_transactions/` - Банкови транзакции
- `currencies/` - Валутни курсове
- `exchange_rates/` - Синхронизация с ECB
- `accounting_periods/` - Отчетни периоди
- `opening_balances/` - Начални салда
- `vat_returns/` - ДДС декларации
- `reports/` - Отчети

#### Integrations & Import/Export
- `saft/` - SAF-T генериране и валидация
- `controlisy/` - Контролизи импорт
- `scanned_invoices/` - Сканирани фактури (OCR)

#### System & Admin
- `admin/` - Админ панели
- `backup/` - Backup & Restore
- `dividends/` - Дивиденти
- `nomenclatures/` - Номенклатури

### Mixins

`actions/mixins/api/auth/` - Reusable логика за аутентикация:
- `require_auth_token.cr` - Изисква валиден JWT токен
- `skip_require_auth_token.cr` - Позволява публичен достъп
- `helpers.cr` - Helper функции за auth

### Директория `backend/src/models/`

Avram модели (Active Record патърн) - 40+ модела:

#### Core Models
- `user.cr` - Потребители
- `user_token.cr` - JWT токени
- `user_company_role.cr` - Роли във фирми
- `company.cr` - Фирми
- `role.cr` - Роли
- `permission.cr` - Права
- `system_setting.cr` - Системни настройки

#### Accounting Models
- `account.cr` - Сметки от сметкоплана
- `counterpart.cr` - Контрагенти
- `invoice.cr` - Фактури
- `invoice_line.cr` - Редове на фактура
- `payment.cr` - Плащания
- `payment_line.cr` - Разпределения на плащания
- `journal_entry.cr` - Счетоводни записи
- `journal_line.cr` - Дебит/Кредит редове
- `accounting_period.cr` - Отчетни периоди
- `opening_balance.cr` - Начални салда

#### Asset & Inventory Models
- `product.cr` - Продукти
- `fixed_asset.cr` - Дълготрайни активи
- `fixed_asset_category.cr` - Категории на активи
- `fixed_asset_transaction.cr` - Движения с ДМА
- `stock_transaction.cr` - Складови операции
- `physical_stock.cr` - Складова наличност
- `warehouse.cr` - Складове

#### Financial Models
- `currency.cr` - Валути
- `exchange_rate.cr` - Валутни курсове
- `bank_account.cr` - Банкови сметки
- `bank_transaction.cr` - Банкови транзакции
- `vat_return.cr` - ДДС декларации
- `vat_journal_entry.cr` - ДДС записи в декларация
- `vat_rate.cr` - ДДС ставки
- `dividend.cr` - Дивиденти
- `shareholder.cr` - Акционери

#### Document & OCR Models
- `document.cr` - Документи
- `scanned_invoice.cr` - Сканирани фактури (OCR)

#### Integration Models
- `controlisy_import.cr` - Импорти от Контролизи
- `combined_nomenclature.cr` - Общата номенклатура (КН)
- `iso_country.cr` - ISO държави
- `iso_currency.cr` - ISO валути

#### SAF-T Nomenclature Models
- `saft_account.cr` - SAF-T сметка
- `saft_tax_type.cr` - Данъчни типове
- `saft_payment_method.cr` - Методи на плащане
- `saft_invoice_type.cr` - Типове фактури
- `saft_unit_of_measure.cr` - Мерни единици
- `saft_asset_account_mapping.cr` - Mapping на активни сметки
- `saft_movement_account_mapping.cr` - Mapping на движения
- `saft_asset_movement_type.cr` - Типове движения на активи
- `saft_stock_movement_type.cr` - Типове складови движения
- `saft_tax_regime.cr` - Данъчни режими
- `saft_region.cr` - Региони
- `saft_iban_format.cr` - IBAN формати
- `saft_product_type.cr` - Типове продукти
- `base_model.cr` - Базов модел

### Директория `backend/src/services/`

Външни услуги и интеграции:

#### SAF-T Services
- `saft_exporter.cr` - SAF-T BG v1.0.1 генериране
  - MasterFiles (Сметкоплан, контрагенти, продукти, активи, банки)
  - GeneralLedgerEntries (Счетоводни записи)
  - SourceDocuments (Фактури, плащания, складови движения)
- `saft_asset_mapper.cr` - Mapping на активи за SAF-T
- `saft_movement_mapper.cr` - Mapping на движения за SAF-T

#### Import/Export Services
- `controlisy_importer.cr` - Контролизи импорт
- `controlisy_parser.cr` - Парсване на Контролизи файлове

#### Exchange Rate Services
- `ecb_exchange_rate_service.cr` - ECB валутни курсове

#### Validation Services
- `vies_service.cr` - VIES валидация на ЕИК/Данъчен номер

#### OCR & AI Services
- `mistral_document_service.cr` - Mistral AI за OCR и разпознаване на фактури

#### System Services
- `email_service.cr` - Email известия
- `database_backup.cr` - Backup & Restore
- `company_accounts_initializer.cr` - Инициализация на сметкоплан
- `period_service.cr` - Управление на отчетни периоди
- `access_control_service.cr` - Контрол на достъпа (RBAC)

## Frontend архитектура

### Leptos Component Architecture

```
App
├── Router
│   ├── Auth Pages
│   │   ├── Login
│   │   ├── Register
│   │   ├── Forgot Password
│   │   ├── Reset Password
│   │   └── Verify Email
│   ├── Main Dashboard
│   ├── Core Accounting Pages
│   │   ├── Invoices Page
│   │   ├── Accounts Page
│   │   ├── Counterparts Page
│   │   ├── Journal Entries Page
│   │   ├── Payments Page
│   │   ├── Accounting Periods
│   │   └── Opening Balances
│   ├── Assets & Inventory Pages
│   │   ├── Products Page
│   │   ├── Fixed Assets Page
│   │   ├── Warehouse Page
│   │   └── Stock Transactions
│   ├── Financial Pages
│   │   ├── Bank Accounts Page
│   │   ├── Bank Transactions Page
│   │   ├── Currencies Page
│   │   ├── Exchange Rates Page
│   │   └── VAT Returns Page
│   ├── Integration Pages
│   │   ├── SAF-T Export Page
│   │   ├── SAF-T Movement Mappings
│   │   ├── Controlisy Import Page
│   │   └── Scanned Invoices Page
│   ├── Management Pages
│   │   ├── Users Page
│   │   ├── Roles Page
│   │   ├── Companies Page
│   │   ├── Documents Page
│   │   └── Reports Page
│   ├── Admin Pages
│   │   ├── Admin Dashboard
│   │   └── System Settings
│   └── User Pages
│       ├── Settings Page
│       └── Profile Page
└── Components (Reusable)
    ├── Table
    ├── Form
    ├── Modal
    ├── Toast
    ├── Pagination
    └── ...
```

### Директория `leptos/src/`

- `main.rs` - Entry point, инициализация
- `app.rs` - Главен App компонент
- `router.rs` - Router конфигурация с routes
- `api.rs` - API клиент (HTTP заявки към backend)
- `models.rs` - TypeScript-like типове за данни
- `context/` - App context (auth, settings, company)
- `constants/` - Константи (API URL, статуси)
- `stores/` - State management (Signal/Store)
- `i18n/` - Преводи (bg.json, en.json)
- `components/` - Reusable UI компоненти
- `pages/` - Страници/Views (30+ страници)

### State Management

Leptos използва:
- `create_signal()` - Reactive state
- `create_rw_signal()` - Read/write reactive state
- `create_resource()` - Async data fetching
- Context API за глобално състояние

## Даннен модел

### Основни Entities

```
User (Потребител)
├── UserTokens (JWT токени)
├── UserCompanyRoles (Роли във фирми)
├── EmailVerificationToken
└── PasswordResetToken

Company (Фирма)
├── Accounts (Сметкоплан)
├── Counterparts (Контрагенти)
│   ├── Customers (Клиенти)
│   └── Suppliers (Доставчици)
├── Products (Продукти)
├── Fixed Assets (Дълготрайни активи)
│   ├── FixedAssetCategories
│   └── FixedAssetTransactions
├── Warehouses (Складове)
├── Invoices (Фактури)
│   └── Invoice Lines (Редове на фактура)
├── Payments (Плащания)
│   └── Payment Lines (Разпределения)
├── Journal Entries (Счетоводни записи)
│   └── Journal Lines (Дебит/Кредит редове)
├── Stock Transactions (Складови движения)
├── Physical Stock (Складова наличност)
├── VAT Returns (ДДС декларации)
│   └── VAT Journal Entries
├── Opening Balances (Начални салда)
├── Accounting Periods (Отчетни периоди)
├── Bank Accounts (Банкови сметки)
│   └── Bank Transactions (Транзакции)
├── Currencies (Валути)
│   └── Exchange Rates (Курсове)
├── Dividends (Дивиденти)
│   └── Shareholders (Акционери)
├── Documents (Документи)
├── Scanned Invoices (Сканирани фактури)
└── System Settings (Системни настройки)

SAF-T Номенклатури
├── SaftAccount
├── SaftTaxType
├── SaftPaymentMethod
├── SaftInvoiceType
├── SaftUnitOfMeasure
├── SaftAssetAccountMapping
├── SaftMovementAccountMapping
├── SaftAssetMovementType
├── SaftStockMovementType
├── SaftTaxRegime
├── SaftRegion
├── SaftIbanFormat
└── SaftProductType
```

## Безопасност

### Аутентикация

- JWT токени за API аутентикация
- Authentic gem за password hashing
- Email verification
- Password reset with token
- Role-based access control (RBAC)
- Permissions за детайлен достъп

### Authorization

- Every API request (except public endpoints) requires valid JWT
- Token validation in `actions/mixins/api/auth/require_auth_token.cr`
- User permissions based on `UserCompanyRole`, `Role`, `Permission`
- Super admin с пълни права

## API Communication

### Request/Response Format

```json
// Request
POST /api/invoices
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json

{
  "invoice": {
    "company_id": 1,
    "counterpart_id": 2,
    "date": "2026-01-30",
    "number": "FA-2026-001",
    "total_amount": 1000.00,
    "vat_amount": 200.00,
    "invoice_lines": [
      {
        "product_id": 1,
        "quantity": 10,
        "unit_price": 100.00,
        "total_price": 1000.00
      }
    ]
  }
}

// Response (Success)
{
  "data": {
    "id": 123,
    "number": "FA-2026-001",
    "date": "2026-01-30",
    "counterpart_name": "ООД Пример",
    "total_amount": "1000.00",
    "vat_amount": "200.00",
    ...
  }
}

// Response (Error)
{
  "errors": {
    "counterpart_id": ["is required"],
    "date": ["invalid format"],
    "invoice_lines": ["cannot be empty"]
  }
}
```

## Интеграции

### SAF-T BG v1.0.1

- Генериране на XML файлове според българската спецификация
- MasterFiles (Сметкоплан, контрагенти, продукти, активи, банки)
- GeneralLedgerEntries (Счетоводни записи)
- SourceDocuments (Фактури, плащания, складови движения, движения с активи)
- Номенклатури за всичко
- Типове отчети: Месечен, При поискване, Годишен

### VIES

- Валидация на VAT номера на EU контрагенти
- API: http://ec.europa.eu/taxation_customs/vies/checkVatService

### ECB Exchange Rates

- Автоматично изтегляне на валутни курсове от Европейската централна банка
- Daily reference rates
- История на курсовете

### Kontrolizi Import

- Импорт на данни от българската система Kontrolizi
- Парсване на специфичен формат на файлове
- Поддържа: контрагенти, продукти, фактури (продажби и покупки), плащания, банкови операции

### Mistral AI (OCR)

- AI-powered OCR и разпознаване на данни от фактури
- Разпознаване на текст от сканирани документи
- Автоматично попълване на полета (доставчик, клиент, суми, ДДС, дати)

### Email Service

- Изпращане на email известия
- Email verification
- Password reset emails
- Възможност за SMTP конфигурация

## Development Workflow

1. **Backend Development**
   ```bash
   cd backend
   lucky dev  # Development server with hot reload
   ```

2. **Frontend Development**
   ```bash
   cd leptos
   trunk serve  # WASM dev server with hot reload
   ```

3. **Database Migrations**
   ```bash
   cd backend
   lucky gen.migration <name>
   lucky db.migrate
   ```

4. **Testing**
   ```bash
   cd backend
   lucky spec  # Run specs
   ```

## Deployment

### Build

```bash
# Backend
cd backend
shards build --release

# Frontend
cd leptos
trunk build --release
```

### Environment

Създайте `.env` файл в `backend/`:
```env
DB_HOST=localhost
DB_PORT=5432
DB_NAME=lucky
DB_USERNAME=postgres
DB_PASSWORD=<password>
LUCKY_ENV=production
SECRET_KEY_BASE=<generate with lucky gen.secret_key>
DATABASE_URL=postgres://user:pass@host:5432/lucky

# Optional: External Services
AZURE_VISION_KEY=<your_key>
AZURE_VISION_ENDPOINT=<your_endpoint>
MISTRAL_API_KEY=<your_key>
```

## Технически детайли

### Crystal/Lucky Version
- Crystal: 1.16.3+
- Lucky: 1.3.0+

### Rust/Leptos Version
- Rust: 1.75+
- Leptos: latest
- Trunk: 0.20+

### Database
- PostgreSQL: 14+
- Avram ORM (Lucky's ORM)

### Libraries
- Backend: Lucky framework ecosystem (authentic, avram, habitat, carbon, etc.)
- Frontend: Leptos, web-sys, wasm-bindgen, gloo-net, serde
- OCR/AI: Mistral AI

---

**Последна актуализация:** 30 Януари 2026
