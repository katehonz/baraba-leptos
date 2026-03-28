# Backend документация

## Технологичен стек

- **Език:** Crystal 1.16.3+
- **Framework:** Lucky Framework 1.3.0+
- **Database:** PostgreSQL 14+
- **ORM:** Avram (Lucky's Active Record)
- **Auth:** Authentic (password hashing + JWT)
- **Email:** Carbon

## Стартиране

### Development mode

```bash
cd backend
shards install
lucky db.create
lucky db.migrate
lucky dev
```

Сървърът ще стартира на `http://localhost:5000`

### Production mode

```bash
cd backend
shards build --release
./bin/app
```

## Конфигурация

### Environment variables

Файл: `backend/.env`

```env
# Database
DB_HOST=localhost
DB_PORT=5432
DB_NAME=lucky
DB_USERNAME=postgres
DB_PASSWORD=your_secure_password_here
DATABASE_URL=postgres://postgres:your_secure_password_here@localhost:5432/lucky

# App
LUCKY_ENV=development
SECRET_KEY_BASE=your_secret_key_base_here

# Session
SESSION_KEY=<session_key>

# External Services (optional)
AZURE_VISION_KEY=<your_key>
AZURE_VISION_ENDPOINT=<your_endpoint>
MISTRAL_API_KEY=<your_key>
```

## Структура на backend

### Директория `src/actions/api/`

Actions са контролерите, които обработват HTTP заявки.

#### Auth & User Management

**Auth:** `actions/api/auth/`
- `signin.cr` - POST `/api/sign_ins` - Вход в системата
- `signup.cr` - POST `/api/sign_ups` - Регистрация на потребител
- `me.cr` - GET `/api/me` - Текущ потребител
- `verify_email.cr` - GET `/api/verify_email` - Валидация на email
- `forgot_password.cr` - POST `/api/forgot_password` - Забравена парола
- `reset_password.cr` - POST `/api/reset_password` - Смяна на парола

**Users:** `actions/api/users/`
- `index.cr` - GET `/api/users` - Списък с потребители
- `show.cr` - GET `/api/users/:id` - Детайли за потребител
- `create.cr` - POST `/api/users` - Създаване на потребител
- `update.cr` - PATCH `/api/users/:id` - Редактиране на потребител
- `delete.cr` - DELETE `/api/users/:id` - Изтриване на потребител

**Roles:** `actions/api/roles/`
- `index.cr` - GET `/api/roles` - Списък с роли
- `show.cr` - GET `/api/roles/:id` - Детайли за роля
- `create.cr` - POST `/api/roles` - Създаване на роля
- `update.cr` - PATCH `/api/roles/:id` - Редактиране на роля
- `delete.cr` - DELETE `/api/roles/:id` - Изтриване на роля
- `assign_permissions.cr` - PUT `/api/roles/:id/permissions` - Задаване на права

#### Core Accounting

**Accounts:** `actions/api/accounts/`
- `index.cr` - GET `/api/accounts` - Списък със сметки
- `show.cr` - GET `/api/accounts/:id` - Детайли за сметка
- `create.cr` - POST `/api/accounts` - Създаване на сметка
- `update.cr` - PATCH `/api/accounts/:id` - Редактиране на сметка
- `delete.cr` - DELETE `/api/accounts/:id` - Изтриване на сметка
- `tree.cr` - GET `/api/accounts/tree` - Сметкоплан като дърво

**Companies:** `actions/api/companies/`
- `index.cr` - GET `/api/companies` - Списък с фирми
- `show.cr` - GET `/api/companies/:id` - Детайли за фирма
- `create.cr` - POST `/api/companies` - Създаване на фирма
- `update.cr` - PATCH `/api/companies/:id` - Редактиране на фирма
- `delete.cr` - DELETE `/api/companies/:id` - Изтриване на фирма
- `initialize_accounts.cr` - POST `/api/companies/:id/initialize_accounts` - Инициализация на сметкоплан

**Counterparts:** `actions/api/counterparts/`
- `index.cr` - GET `/api/counterparts` - Списък с контрагенти
- `show.cr` - GET `/api/counterparts/:id` - Детайли за контрагент
- `create.cr` - POST `/api/counterparts` - Създаване на контрагент
- `update.cr` - PATCH `/api/counterparts/:id` - Редактиране на контрагент
- `delete.cr` - DELETE `/api/counterparts/:id` - Изтриване на контрагент

**Invoices:** `actions/api/invoices/`
- `index.cr` - GET `/api/invoices` - Списък с фактури
- `show.cr` - GET `/api/invoices/:id` - Детайли за фактура
- `create.cr` - POST `/api/invoices` - Създаване на фактура
- `update.cr` - PATCH `/api/invoices/:id` - Редактиране на фактура
- `delete.cr` - DELETE `/api/invoices/:id` - Изтриване на фактура

**Payments:** `actions/api/payments/`
- `index.cr` - GET `/api/payments` - Списък с плащания
- `show.cr` - GET `/api/payments/:id` - Детайли за плащане
- `create.cr` - POST `/api/payments` - Създаване на плащане
- `update.cr` - PATCH `/api/payments/:id` - Редактиране на плащане
- `delete.cr` - DELETE `/api/payments/:id` - Изтриване на плащане

**Journal Entries:** `actions/api/journal_entries/`
- `index.cr` - GET `/api/journal_entries` - Списък със счетоводни записи
- `show.cr` - GET `/api/journal_entries/:id` - Детайли за запис
- `create.cr` - POST `/api/journal_entries` - Създаване на запис
- `update.cr` - PATCH `/api/journal_entries/:id` - Редактиране на запис
- `delete.cr` - DELETE `/api/journal_entries/:id` - Изтриване на запис

**Accounting Periods:** `actions/api/accounting_periods/`
- `index.cr` - GET `/api/accounting_periods` - Списък с отчетни периоди
- `show.cr` - GET `/api/accounting_periods/:id` - Детайли за период
- `create.cr` - POST `/api/accounting_periods` - Създаване на период
- `update.cr` - PATCH `/api/accounting_periods/:id` - Редактиране на период
- `delete.cr` - DELETE `/api/accounting_periods/:id` - Изтриване на период
- `close.cr` - POST `/api/accounting_periods/:id/close` - Затваряне на период

**Opening Balances:** `actions/api/opening_balances/`
- `index.cr` - GET `/api/opening_balances` - Списък с начални салда
- `create.cr` - POST `/api/opening_balances` - Създаване на начално салдо
- `update.cr` - PATCH `/api/opening_balances/:id` - Редактиране на начално салдо
- `delete.cr` - DELETE `/api/opening_balances/:id` - Изтриване на начално салдо

#### Assets & Inventory

**Products:** `actions/api/products/`
- `index.cr` - GET `/api/products` - Списък с продукти
- `show.cr` - GET `/api/products/:id` - Детайли за продукт
- `create.cr` - POST `/api/products` - Създаване на продукт
- `update.cr` - PATCH `/api/products/:id` - Редактиране на продукт
- `delete.cr` - DELETE `/api/products/:id` - Изтриване на продукт

**Fixed Assets:** `actions/api/fixed_assets/`
- `index.cr` - GET `/api/fixed_assets` - Списък с ДМА
- `show.cr` - GET `/api/fixed_assets/:id` - Детайли за ДМА
- `create.cr` - POST `/api/fixed_assets` - Създаване на ДМА
- `update.cr` - PATCH `/api/fixed_assets/:id` - Редактиране на ДМА
- `delete.cr` - DELETE `/api/fixed_assets/:id` - Изтриване на ДМА
- `calculate_depreciation.cr` - POST `/api/fixed_assets/:id/calculate_depreciation` - Изчисляване на амортизация

**Fixed Asset Categories:** `actions/api/fixed_asset_categories/`
- `index.cr` - GET `/api/fixed_asset_categories` - Списък с категории
- `create.cr` - POST `/api/fixed_asset_categories` - Създаване на категория
- `update.cr` - PATCH `/api/fixed_asset_categories/:id` - Редактиране на категория
- `delete.cr` - DELETE `/api/fixed_asset_categories/:id` - Изтриване на категория

#### Financial & Reporting

**Bank Accounts:** `actions/api/bank_accounts/`
- `index.cr` - GET `/api/bank_accounts` - Списък с банкови сметки
- `show.cr` - GET `/api/bank_accounts/:id` - Детайли за сметка
- `create.cr` - POST `/api/bank_accounts` - Създаване на сметка
- `update.cr` - PATCH `/api/bank_accounts/:id` - Редактиране на сметка
- `delete.cr` - DELETE `/api/bank_accounts/:id` - Изтриване на сметка

**Bank Transactions:** `actions/api/bank_transactions/`
- `index.cr` - GET `/api/bank_transactions` - Списък с транзакции (+ `is_allocated`, `journal_lines`)
- `show.cr` - GET `/api/bank_transactions/:id` - Детайли за транзакция
- `create.cr` - POST `/api/bank_transactions` - Създаване на транзакция
- `update.cr` - PATCH `/api/bank_transactions/:id` - Редактиране на транзакция
- `delete.cr` - DELETE `/api/bank_transactions/:id` - Изтриване на транзакция
- `import.cr` - POST `/api/bank_transactions/import` - Импорт от банков файл (+ auto-journal с буферна сметка)
- `reallocate.cr` - POST `/api/bank_transactions/:id/reallocate` - Преразпределяне от буферна към реална сметка
- `match_invoices.cr` - POST `/api/bank_transactions/:id/match_invoices` - Съвпадане с фактури

**Currencies:** `actions/api/currencies/`
- `index.cr` - GET `/api/currencies` - Списък с валути
- `create.cr` - POST `/api/currencies` - Създаване на валута
- `update.cr` - PATCH `/api/currencies/:id` - Редактиране на валута
- `delete.cr` - DELETE `/api/currencies/:id` - Изтриване на валута

**Exchange Rates:** `actions/api/exchange_rates/`
- `index.cr` - GET `/api/exchange_rates` - Списък с валутни курсове
- `show.cr` - GET `/api/exchange_rates/:id` - Детайли за курс
- `create.cr` - POST `/api/exchange_rates` - Създаване на курс
- `update.cr` - PATCH `/api/exchange_rates/:id` - Редактиране на курс
- `delete.cr` - DELETE `/api/exchange_rates/:id` - Изтриване на курс
- `sync_ecb.cr` - POST `/api/exchange_rates/sync_ecb` - Синхронизация с ECB

**VAT Returns:** `actions/api/vat_returns/`
- `index.cr` - GET `/api/vat_returns` - Списък с ДДС декларации
- `show.cr` - GET `/api/vat_returns/:id` - Детайли за декларация
- `create.cr` - POST `/api/vat_returns` - Създаване на декларация
- `update.cr` - PATCH `/api/vat_returns/:id` - Редактиране на декларация
- `delete.cr` - DELETE `/api/vat_returns/:id` - Изтриване на декларация
- `calculate.cr` - POST `/api/vat_returns/calculate` - Изчисляване на ДДС
- `export_xml.cr` - GET `/api/vat_returns/:id/export_xml` - Експорт в XML

**Reports:** `actions/api/reports/`
- `index.cr` - GET `/api/reports` - Списък с отчети
- `balance_sheet.cr` - GET `/api/reports/balance_sheet` - Балансов отчет
- `profit_loss.cr` - GET `/api/reports/profit_loss` - Отчет за печалба и загуба
- `trial_balance.cr` - GET `/api/reports/trial_balance` - Пробен баланс

**Dividends:** `actions/api/dividends/`
- `index.cr` - GET `/api/dividends` - Списък с дивиденти
- `create.cr` - POST `/api/dividends` - Създаване на дивидент
- `update.cr` - PATCH `/api/dividends/:id` - Редактиране на дивидент
- `delete.cr` - DELETE `/api/dividends/:id` - Изтриване на дивидент

#### Integrations & Import/Export

**SAF-T:** `actions/api/saft/`
- `validate.cr` - GET `/api/saft/validate` - Валидация на SAF-T данни
- `export.cr` - GET `/api/saft/export` - Експорт на SAF-T XML
- `movement_mappings.cr` - GET/POST `/api/saft/movement_mappings` - SAF-T mappings за движения
- `update_movement_mapping.cr` - PATCH `/api/saft/movement_mappings/:id` - Редактиране на mapping

**Controlisy:** `actions/api/controlisy/`
- `import.cr` - POST `/api/controlisy/import` - Импорт от Контролизи
- `status.cr` - GET `/api/controlisy/imports/:id` - Статус на импорт

**Scanned Invoices:** `actions/api/scanned_invoices/`
- `index.cr` - GET `/api/scanned_invoices` - Списък със сканирани фактури
- `upload.cr` - POST `/api/scanned_invoices/upload` - Качване на файл
- `process_mistral.cr` - POST `/api/scanned_invoices/:id/process_mistral` - Обработка с Mistral AI
- `create_invoice.cr` - POST `/api/scanned_invoices/:id/create_invoice` - Създаване на фактура

**Documents:** `actions/api/documents/`
- `index.cr` - GET `/api/documents` - Списък с документи
- `show.cr` - GET `/api/documents/:id` - Детайли за документ
- `upload.cr` - POST `/api/documents/upload` - Качване на документ
- `download.cr` - GET `/api/documents/:id/download` - Сваляне на документ
- `delete.cr` - DELETE `/api/documents/:id` - Изтриване на документ

#### System & Admin

**Admin:** `actions/api/admin/`
- `index.cr` - GET `/api/admin` - Админ панел
- `users.cr` - GET `/api/admin/users` - Управление на потребители
- `companies.cr` - GET `/api/admin/companies` - Управление на фирми
- `settings.cr` - GET `/api/admin/settings` - Системни настройки

**Backup:** `actions/api/backup/`
- `create.cr` - POST `/api/backup/create` - Създаване на бекъп
- `list.cr` - GET `/api/backup/list` - Списък с бекъпи
- `restore.cr` - POST `/api/backup/restore` - Възстановяване от бекъп
- `download.cr` - GET `/api/backup/:id/download` - Сваляне на бекъп

**Nomenclatures:** `actions/api/nomenclatures/`
- `combined_nomenclatures.cr` - GET `/api/nomenclatures/combined_nomenclatures` - Обща номенклатура

#### Other

**Settings:** `actions/api/settings/`
- `index.cr` - GET `/api/settings` - Настройки на потребителя
- `update.cr` - PATCH `/api/settings` - Обновяване на настройки

**Profile:** `actions/api/profile/`
- `show.cr` - GET `/api/profile` - Профил
- `update.cr` - PATCH `/api/profile` - Обновяване на профил

#### Mixins

`actions/mixins/api/auth/`
- `require_auth_token.cr` - Изисква валиден JWT токен
- `skip_require_auth_token.cr` - Позволява публичен достъп
- `helpers.cr` - Helper функции за auth

Използване:
```crystal
class Api::Invoices::Index < ApiAction
  include Api::Auth::RequireAuthToken  # Защита с JWT

  get "/api/invoices" do
    invoices = InvoiceQuery.new
    json InvoicesSerializer.new(invoices)
  end
end
```

### Директория `src/models/`

Avram модели (Active Record патърн) - 40+ модела.

#### User Management

**User** (`user.cr`)
- Потребители в системата
- Relations: `user_tokens`, `user_company_roles`

**UserToken** (`user_token.cr`)
- JWT токени за аутентикация
- Свързани с User

**UserCompanyRole** (`user_company_role.cr`)
- Роли на потребители във фирми
- Relations: `user`, `company`, `role`

**Role** (`role.cr`)
- Роли (admin, accountant, viewer)
- Relations: `permissions`

**Permission** (`permission.cr`)
- Права за достъп
- Relations: `roles`

**SystemSetting** (`system_setting.cr`)
- Системни настройки

#### Core Accounting Models

**Company** (`company.cr`)
- Фирми/компании
- Relations: `accounts`, `invoices`, `journal_entries`, `products`, `counterparts`, `fixed_assets`

**Account** (`account.cr`)
- Сметки от сметкоплана
- Полета: `code`, `name`, `account_type`, `is_active`, `tracks_articles` (отчитане на артикули/продукти)
- SAF-T полета: `saft_account_id`, `taxpayer_account_id`, `grouping_category`, `grouping_code`, `account_creation_date`
- Салда: `opening_debit_balance`, `opening_credit_balance`, `closing_debit_balance`, `closing_credit_balance`
- Relations: `journal_lines`, `saft_account`, `counterparts`, `payment_lines`

**Counterpart** (`counterpart.cr`)
- Контрагенти (клиенти и доставчици)
- Relations: `invoices`, `payments`

**Invoice** (`invoice.cr`)
- Фактури (изходящи и входящи)
- Relations: `invoice_lines`, `payments`

**InvoiceLine** (`invoice_line.cr`)
- Редове от фактура
- Relations: `invoice`, `product`

**Payment** (`payment.cr`)
- Плащания
- Relations: `payment_lines`, `invoice`

**PaymentLine** (`payment_line.cr`)
- Разпределения на плащания

**JournalEntry** (`journal_entry.cr`)
- Счетоводни записи
- Relations: `journal_lines`

**JournalLine** (`journal_line.cr`)
- Редове на счетоводен запис (дебит/кредит)
- Relations: `journal_entry`, `account`

**AccountingPeriod** (`accounting_period.cr`)
- Отчетни периоди
- Relations: `journal_entries`

**OpeningBalance** (`opening_balance.cr`)
- Начални салда

#### Asset & Inventory Models

**Product** (`product.cr`)
- Продукти и услуги
- Relations: `invoice_lines`, `stock_transactions`

**FixedAsset** (`fixed_asset.cr`)
- Дълготрайни материални активи
- Relations: `fixed_asset_transactions`

**FixedAssetCategory** (`fixed_asset_category.cr`)
- Категории на ДМА
- Relations: `fixed_assets`

**FixedAssetTransaction** (`fixed_asset_transaction.cr`)
- Движения с ДМА (придобиване, амортизация, продажба)
- Relations: `fixed_asset`, `journal_entry`

**Warehouse** (`warehouse.cr`)
- Складове
- Relations: `stock_transactions`, `physical_stock`

**StockTransaction** (`stock_transaction.cr`)
- Складови движения (приемане, отпускане, трансфер)
- Relations: `product`, `warehouse`

**PhysicalStock** (`physical_stock.cr`)
- Складова наличност
- Relations: `product`, `warehouse`

#### Financial Models

**Currency** (`currency.cr`)
- Валути
- Relations: `exchange_rates`

**ExchangeRate** (`exchange_rate.cr`)
- Валутни курсове
- Relations: `currency`

**BankAccount** (`bank_account.cr`)
- Банкови сметки
- Полета: `name`, `bank_name`, `iban`, `bic`, `currency`
- Relations: `bank_transactions`, `gl_account : Account?`, `buffer_account : Account?`

**BankTransaction** (`bank_transaction.cr`)
- Банкови транзакции
- Полета: `date`, `amount`, `currency`, `description`, `contra_account`, `contra_name`, `reference`
- Relations: `bank_account`, `journal_entry : JournalEntry?`

**VatRate** (`vat_rate.cr`)
- ДДС ставки

**VATReturn** (`vat_return.cr`)
- ДДС декларации
- Relations: `vat_journal_entries`

**VATJournalEntry** (`vat_journal_entry.cr`)
- ДДС записи в декларация
- Relations: `vat_return`

**Dividend** (`dividend.cr`)
- Дивиденти

**Shareholder** (`shareholder.cr`)
- Акционери
- Relations: `dividends`

#### Document & OCR Models

**Document** (`document.cr`)
- Документи (файлове)

**ScannedInvoice** (`scanned_invoice.cr`)
- Сканирани фактури (OCR)
- Relations: `document`

#### Integration Models

**ControlisyImport** (`controlisy_import.cr`)
- Импорти от Контролизи

**CombinedNomenclature** (`combined_nomenclature.cr`)
- Общата номенклатура (КН)

**IsoCountry** (`iso_country.cr`)
- ISO държави

**IsoCurrency** (`iso_currency.cr`)
- ISO валути

#### SAF-T Models

**SaftCashAccountMapping** (`saft_cash_account_mapping.cr`)
- Mapping на парични кореспонденции за SAF-T (каса/банка)
- Полета: `cash_movement_type`, `debit_account`, `credit_account`, wildcard patterns

**SaftAccount** (`saft_account.cr`)
- SAF-T стандартна сметка от НАП номенклатура (НРА_Ном_Сметки)
- Полета: `code`, `name`, `section_code`, `section_name`, `group_code`, `group_name`

**SaftTaxType** (`saft_tax_type.cr`)
- Данъчни типове (ДДС ставки)

**SaftPaymentMethod** (`saft_payment_method.cr`)
- Методи на плащане

**SaftInvoiceType** (`saft_invoice_type.cr`)
- Типове фактури

**SaftUnitOfMeasure** (`saft_unit_of_measure.cr`)
- Мерни единици (UN/ECE)

**SaftAssetAccountMapping** (`saft_asset_account_mapping.cr`)
- Mapping на активи за SAF-T

**SaftMovementAccountMapping** (`saft_movement_account_mapping.cr`)
- Mapping на движения за SAF-T

**SaftAssetMovementType** (`saft_asset_movement_type.cr`)
- Типове движения на активи

**SaftStockMovementType** (`saft_stock_movement_type.cr`)
- Типове складови движения

**SaftTaxRegime** (`saft_tax_regime.cr`)
- Данъчни режими

**SaftRegion** (`saft_region.cr`)
- Региони

**SaftIbanFormat** (`saft_iban_format.cr`)
- IBAN формати

**SaftProductType** (`saft_product_type.cr`)
- Типове продукти

**BaseModel** (`base_model.cr`)
- Базов модел с common функционалност

### Директория `src/operations/`

Operations съдържат бизнес логика. Те изолират логиката от actions.

#### Пример за operation

```crystal
# src/operations/invoices/create_invoice.cr
class Invoices::CreateInvoice < Invoice::SaveOperation
  permit_columns date, number, counterpart_id, total_amount, vat_amount, notes

  before_save do
    validate_unique_number
    calculate_totals
  end

  def validate_unique_number
    if InvoiceQuery.new.number(number.value).company_id(company_id.value).any?
      number.add_error("must be unique")
    end
  end

  def calculate_totals
    # Логика за изчисляване на суми
  end
end
```

Използване в action:
```crystal
class Api::Invoices::Create < ApiAction
  include Api::Auth::RequireAuthToken

  post "/api/invoices" do
    invoice = Invoices::CreateInvoice.create(params)
    if invoice.valid?
      json InvoiceSerializer.new(invoice.record)
    else
      json ErrorsSerializer.new(invoice), status: 422
    end
  end
end
```

#### Available Operations

`operations/accounts/` - `create_account.cr`, `update_account.cr`
`operations/companies/` - `create_company.cr`, `update_company.cr`
`operations/counterparts/` - `create_counterpart.cr`, `update_counterpart.cr`
`operations/invoices/` - `create_invoice.cr`, `update_invoice.cr`
`operations/journal_entries/` - `create_journal_entry.cr`, `update_journal_entry.cr`
`operations/payments/` - `create_payment.cr`, `update_payment.cr`
`operations/products/` - `create_product.cr`, `update_product.cr`
`operations/fixed_assets/` - `create_fixed_asset.cr`, `update_fixed_asset.cr`
`operations/opening_balances/` - `create_opening_balance.cr`

### Директория `src/queries/`

Avram Query objects за сложни заявки.

#### Пример

```crystal
# src/queries/invoice_query.cr
class InvoiceQuery < Invoice::BaseQuery
  def for_company(company_id : Int64)
    company_id(company_id)
  end

  def for_period(start_date : Time, end_date : Time)
    date.gte(start_date).and.date.lte(end_date)
  end

  def for_counterpart(counterpart_id : Int64)
    counterpart_id(counterpart_id)
  end

  def paid
    paid_at.not_nil
  end

  def unpaid
    paid_at.nil
  end
end
```

Използване:
```crystal
# В action
invoices = InvoiceQuery.new
  .for_company(current_user.company_id)
  .for_period(start_date, end_date)
  .paid
```

### Директория `src/services/`

Сървиси за външни интеграции.

#### SAF-T Services

**SaftExporter** (`saft_exporter.cr`)

Генерира SAF-T BG v1.0.1 XML файлове.

Методи:
- `generate : String` - Генерира пълен SAF-T XML
- `validate(company_id : Int64) : Array(String)` - Валидация

Генерира:
- MasterFiles (Сметкоплан, контрагенти, продукти, активи, банки)
- GeneralLedgerEntries (Счетоводни записи)
- SourceDocuments (Фактури, плащания, складови движения, движения с активи)

**SaftAssetMapper** (`saft_asset_mapper.cr`)

Mapping на активи за SAF-T.

Методи:
- `map_to_saft(fixed_asset : FixedAsset) : Hash` - Конвертира актив в SAF-T формат

**SaftMovementMapper** (`saft_movement_mapper.cr`)

Mapping на движения за SAF-T.

Методи:
- `map_transaction(transaction : StockTransaction) : Hash`
- `map_asset_transaction(transaction : FixedAssetTransaction) : Hash`

#### Import/Export Services

**ControlisyImporter** (`controlisy_importer.cr`)

Импорт на данни от българската система Kontrolizi.

Методи:
- `import(file_path : String, company_id : Int64) : ControlisyImport`
- `parse(file_path : String) : Hash`

**ControlisyParser** (`controlisy_parser.cr`)

Парсване на Kontrolizi файлове.

Методи:
- `parse_headers(file_content : String) : Hash`
- `parse_rows(file_content : String) : Array(Hash)`
- `parse_summary(file_content : String) : Hash`

#### Exchange Rate Services

**EcbExchangeRateService** (`ecb_exchange_rate_service.cr`)

Изтегля валутни курсове от Европейската централна банка.

Методи:
- `fetch_daily_rates : Hash(String, Float64)`
- `sync_to_database(company_id : Int64) : Int32`

API: `https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml`

#### Validation Services

**ViesService** (`vies_service.cr`)

Валидация на VAT номера на EU контрагенти.

Методи:
- `validate(vat_number : String, country_code : String) : ViesResult`

API: `http://ec.europa.eu/taxation_customs/vies/checkVatService`

#### OCR & AI Services

**MistralDocumentService** (`mistral_document_service.cr`)

AI-powered OCR и разпознаване на фактури с Mistral AI.

Методи:
- `extract_invoice_data(text : String) : Hash`
- `is_configured? : Bool`

#### System Services

**EmailService** (`email_service.cr`)

Изпращане на email известия.

Методи:
- `send_verification_email(user : User) : Bool`
- `send_password_reset(user : User, token : String) : Bool`
- `send_invoice_notification(invoice : Invoice) : Bool`

**DatabaseBackup** (`database_backup.cr`)

Backup & Restore.

Методи:
- `create_backup : String` - Връща пътя до бекъп файла
- `restore_backup(backup_path : String) : Bool`
- `list_backups : Array(String)`

**CompanyAccountsInitializer** (`company_accounts_initializer.cr`)

Инициализация на сметкоплан за нова фирма.

Методи:
- `initialize(company : Company) : Array(Account)`

**PeriodService** (`period_service.cr`)

Управление на отчетни периоди.

Методи:
- `open_period(company_id : Int64, date : Time) : AccountingPeriod`
- `close_period(company_id : Int64, period_id : Int64) : Bool`
- `is_period_open?(company_id : Int64, date : Time) : Bool`

**AccessControlService** (`access_control_service.cr`)

Контрол на достъпа (RBAC).

Методи:
- `has_permission?(user : User, permission : String) : Bool`
- `has_role?(user : User, role : String) : Bool`
- `get_permissions(user : User) : Array(Permission)`

### Директория `src/serializers/`

JSON serialization.

#### Пример

```crystal
class InvoiceSerializer < Lucky::Serializer
  def initialize(@invoice : Invoice)
  end

  def render
    {
      id: @invoice.id,
      number: @invoice.number,
      date: @invoice.date.to_s("%Y-%m-%d"),
      counterpart_id: @invoice.counterpart_id,
      counterpart_name: @invoice.counterpart.name,
      total_amount: @invoice.total_amount.to_s,
      vat_amount: @invoice.vat_amount.to_s,
      paid: @invoice.paid_at != nil,
      created_at: @invoice.created_at.to_s("%Y-%m-%d %H:%M:%S")
    }
  end
end
```

### Директория `src/handlers/`

Custom middleware.

### Директория `src/emails/`

Email templates (Lucky::Email).

## Database Migrations

Миграциите се намират в `backend/db/migrations/`.

### Създаване на миграция

```bash
cd backend
lucky gen.migration AddFieldToTable
```

### Изпълнение на миграции

```bash
lucky db.migrate
```

### Връщане назад

```bash
lucky db.rollback
```

### Премахване на базата

```bash
lucky db.drop
```

## Тестване

Тестовете се намират в `backend/spec/`.

### Стартиране на тестове

```bash
cd backend
lucky spec
```

### Създаване на spec

```bash
lucky gen.model_spec ModelName
lucky gen.action_spec ActionName
```

## Полезни команди

```bash
# Генериране на model
lucky gen.model ModelName

# Генериране на action
lucky gen.action Api::Resource::Create

# Генериране на operation
lucky gen.operation Resource::SaveOperation

# Генериране на migration
lucky gen.migration AddField

# Development server
lucky dev

# Production build
shards build --release

# Database
lucky db.create
lucky db.migrate
lucky db.rollback
lucky db.drop

# Testing
lucky spec

# Generate secret key
lucky gen.secret_key
```

## Error Handling

Lucky има вграден error handling.

### Custom Errors

Можете да създавате custom errors в `src/actions/errors/`.

Пример:
```crystal
class Api::Errors::Show < ApiAction
  get "/api/errors/:code" do
    code = route_params.code
    json({ error: "Error occurred", code: code }), status: 400
  end
end
```

## Logging

Lucky log-ва към STDOUT.

В production, използвайте logging tool като:
- Papertrail
- LogDNA
- Datadog

## Performance

### Database Indexes

Уверете се, че имате appropriate indexes на foreign keys и често търсени полета.

### Caching

За caching може да използвате:
- LuckyCache shard
- Redis
- Memcached

## Security

### Best Practices

1. Винаги валидирайте входните данни (operations)
2. Използвайте `include Api::Auth::RequireAuthToken` за защитени endpoints
3. Използвайте параметризирани заявки (Avram query objects)
4. Никога не връщайте чувствителни данни (пароли, токени)
5. Използвайте HTTPS в production
6. Валидирайте user permissions преди action
7. Използвайте prepared statements (Avram го прави автоматично)

### Rate Limiting

Може да се добави rate limiting middleware.

### CORS

За CORS настройките вижте `config/server.cr`.

---

**Последна актуализация:** 14 Февруари 2026
