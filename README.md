# Baraba

![Baraba Screenshot](baraba-online.png)

**Full-stack счетоводно приложение с Crystal (Lucky Framework) backend и Leptos/Rust WASM frontend**

---

## ⚠️ ВАЖНО

> **Това е работещо приложение в продукшън**, което вади ДДС за НАП. Основните операции са импортирани и проверени със **Софтуера на НАП**, но не сме имали възможност да тестваме абсолютно всичко.

> **Не поддържаме този репозиторий!** Приложението работи с лични данни и фактури. Преди всяко публикуване трием личните данни. Затова нашите проекти имат малко комити и нямат "дърво".

> **Единственият ви начин:** Направете **fork**, ако сте корави и смели, и продължете по своя път.

---

## Структура на проекта

```
baraba-2/
├── backend/              # Crystal / Lucky Framework API
│   ├── config/           # Конфигурация
│   │   ├── database.cr   # База данни
│   │   └── server.cr     # Сървър настройки
│   ├── src/
│   │   ├── actions/      # API endpoints
│   │   │   └── api/      # REST API
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
│   │   │       ├── users/
│   │   │       └── ...
│   │   ├── models/       # Data models (40+ модела)
│   │   ├── operations/   # Business logic
│   │   ├── queries/      # Database queries
│   │   ├── services/     # Business services
│   │   └── serializers/  # JSON serialization
│   ├── db/migrations/    # Миграции
│   ├── shard.yml         # Crystal dependencies
│   └── .env              # Environment variables
│
├── leptos/               # Leptos Rust WASM frontend
│   ├── src/
│   │   ├── main.rs       # Entry point
│   │   ├── api.rs        # API клиент
│   │   ├── router.rs     # Routing
│   │   ├── context/      # App context
│   │   ├── constants/    # Константи
│   │   ├── components/   # UI компоненти
│   │   ├── pages/        # 30+ страници
│   │   └── stores/       # State management
│   ├── Cargo.toml        # Rust dependencies
│   ├── Trunk.toml        # Trunk build config
│   └── index.html        # Entry HTML
│
├── docs/                 # Документация
├── old/                  # Стара версия (използване само за референция)
├── start_local.sh        # Стартиращ скрипт
├── stop_local.sh         # Спиращ скрипт
└── README.md             # Този файл
```

---

## Изисквания

- **Crystal** >= 1.16.3
- **Lucky CLI**
- **Rust** >= 1.75
- **Trunk** (WASM bundler)
- **PostgreSQL** 14+
- **Node.js** (за някои build инструменти)

### Инсталация (Ubuntu/Debian)

```bash
# PostgreSQL
sudo apt install postgresql postgresql-contrib

# Crystal
curl -fsSL https://crystal-lang.org/install.sh | sudo bash

# Lucky CLI
git clone https://github.com/luckyframework/lucky_cli
cd lucky_cli && shards build --release
sudo mv bin/lucky /usr/local/bin/

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Trunk (WASM bundler)
cargo install trunk
rustup target add wasm32-unknown-unknown
```

---

## Конфигурация на базата данни

```
Хост:       localhost
Порт:       5432
База:       lucky
Потребител: postgres
Парола:     your_password_here
```

### Създаване на базата

```bash
sudo -u postgres psql
CREATE DATABASE lucky;
ALTER USER postgres WITH PASSWORD 'your_password_here';
\q
```

---

## Бърз старт

```bash
cd /home/dvg/new-git/baraba-2

# Стартиране
./start_local.sh

# Спиране
./stop_local.sh
```

---

## Ръчно стартиране

### Backend (Lucky Framework)

```bash
cd backend
shards install
lucky db.create
lucky db.migrate
lucky dev
```

**Backend URL:** http://localhost:5000

### Frontend (Leptos/Rust)

```bash
cd leptos
trunk serve
```

**Frontend URL:** http://localhost:8080

---

## Frontend страници

| Страница | URL | Описание |
|----------|-----|----------|
| Login | `/login` | Вход в системата |
| Register | `/register` | Регистрация |
| Forgot Password | `/forgot-password` | Забравена парола |
| Reset Password | `/reset-password` | Смяна на парола |
| Verify Email | `/verify-email` | Валидация на email |
| Dashboard | `/` | Табло с статистики |
| Invoices | `/invoices` | Фактури |
| Accounts | `/accounts` | Сметкоплан |
| Counterparts | `/counterparts` | Контрагенти |
| Products | `/products` | Продукти |
| Fixed Assets | `/fixed-assets` | Дълготрайни активи |
| Users | `/users` | Потребители |
| Currencies | `/currencies` | Валути (ECB курсове) |
| Exchange Rates | `/exchange-rates` | Валутни курсове |
| Bank Accounts | `/bank-accounts` | Банкови сметки |
| Bank Transactions | `/bank-transactions` | Банкови транзакции |
| Journal Entries | `/journal-entries` | Счетоводни записи |
| Payments | `/payments` | Плащания |
| VAT Returns | `/vat-returns` | ДДС декларации |
| Accounting Periods | `/accounting-periods` | Отчетни периоди |
| Opening Balances | `/opening-balances` | Начални салда |
| Warehouse | `/warehouse` | Складово управление |
| Stock Transactions | `/stock-transactions` | Складови движения |
| Documents | `/documents` | Документи |
| Scanned Invoices | `/scanned-invoices` | Сканирани фактури (OCR) |
| SAF-T Export | `/saft-export` | SAF-T BG експорт |
| Controlisy Import | `/controlisy-import` | Импорт от Контролизи |
| Reports | `/reports` | Отчети |
| Settings | `/settings` | Настройки |
| Profile | `/profile` | Профил |
| Roles | `/roles` | Роли и права |
| Admin Dashboard | `/admin` | Админ панел |
| System Settings | `/system-settings` | Системни настройки |

---

## API Endpoints

### Auth
| Метод | URL | Описание |
|-------|-----|----------|
| POST | `/api/sign_ups` | Регистрация |
| POST | `/api/sign_ins` | Вход |
| GET | `/api/me` | Текущ потребител |
| POST | `/api/forgot_password` | Забравена парола |
| POST | `/api/reset_password` | Смяна на парола |
| GET | `/api/verify_email` | Валидация на email |

### Health & System
| Метод | URL | Описание |
|-------|-----|----------|
| GET | `/api/health` | Health check |
| GET | `/api/system_settings` | Системни настройки |
| PUT | `/api/system_settings` | Обновяване на настройки |

### SAF-T
| Метод | URL | Описание |
|-------|-----|----------|
| GET | `/api/saft/validate` | SAF-T валидация |
| GET | `/api/saft/export` | SAF-T XML експорт |
| GET | `/api/saft/movement_mappings` | SAF-T mappings за движения |

### Integrations
| Метод | URL | Описание |
|-------|-----|----------|
| POST | `/api/controlisy/import` | Импорт от Контролизи |
| GET | `/api/currencies/sync_ecb` | Синхронизация с ECB |
| POST | `/api/vies/validate` | VIES валидация |
| POST | `/api/scanned_invoices/upload` | Качване на сканирана фактура |
| POST | `/api/scanned_invoices/process_mistral` | Обработка с Mistral AI |

### Backup & Restore
| Метод | URL | Описание |
|-------|-----|----------|
| GET | `/api/backup/create` | Създаване на бекъп |
| GET | `/api/backup/list` | Списък с бекъпи |
| POST | `/api/backup/restore` | Възстановяване от бекъп |

---

## Основни функционалности

### Счетоводство
- ✅ Пълен счетоводен модел (40+ таблици)
- ✅ Сметкоплан
- ✅ Счетоводни записи (дебит/кредит)
- ✅ Начални салда
- ✅ Отчетни периоди
- ✅ Контрагенти (клиенти/доставчици)
- ✅ Фактури (изходящи/входящи)
- ✅ Плащания и разпределения
- ✅ ДДС декларации

### Дълготрайни активи
- ✅ Регистър на ДМА
- ✅ Амортизация
- ✅ Движения (придобиване, продажба, ликвидация)
- ✅ Категории на активи

### Складово управление
- ✅ Складове
- ✅ Продукти и услуги
- ✅ Складови движения
- ✅ Складова наличност
- ✅ Единични счетоводни цени

### Валутно управление
- ✅ Множество валути
- ✅ Автоматично изтегляне от ECB
- ✅ История на курсовете
- ✅ Вализиране на чуждестранни фактури

### Интеграции
- ✅ **SAF-T BG v1.0.1** - Пълна поддръжка
- ✅ **Контролизи** - Импорт на данни
- ✅ **VIES** - Валидация на EU VAT номера
- ✅ **ECB** - Валутни курсове

### OCR & AI
- ✅ **Mistral AI** - OCR и разпознаване на данни от фактури
- ✅ Сканиране и автоматично попълване

### Други
- ✅ Банкови сметки и транзакции
- ✅ Дивиденти и акционери
- ✅ Обща номенклатура (КН)
- ✅ Email известия
- ✅ Backup & Restore
- ✅ Разширени права и роли
- ✅ Многоезичен интерфейс (BG/EN)

---

## SAF-T BG v1.0.1

Приложението поддържа генериране на SAF-T файлове съгласно българските изисквания:

- **MasterFiles:** Сметкоплан, клиенти, доставчици, продукти, ДМА, банки, данъчни настройки
- **GeneralLedgerEntries:** Счетоводни записи
- **SourceDocuments:** Фактури, плащания, складови движения, транзакции с активи
- **Номенклатури:** Данъчни кодове, видове плащания, мерни единици (UN/ECE), видове движения

### Типове отчети
- **Месечен** - Standard SAF-T отчет
- **При поискване** - Материални запаси
- **Годишен** - Дълготрайни активи

---

## Environment Variables

**Файл:** `backend/.env`

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

---

## Полезни команди

```bash
# Backend
cd backend
shards install          # Инсталиране на зависимости
lucky dev               # Development server
lucky db.create         # Създаване на база
lucky db.migrate        # Миграции
lucky db.rollback       # Връщане назад
lucky spec              # Тестове
lucky gen.model ModelName
lucky gen.action Api::Resource::Create
lucky gen.migration AddField

# Frontend
cd leptos
trunk serve             # Development server with hot reload
trunk build --release   # Production build

# Спиране на всичко
./stop_local.sh
```

---

## Технологии

### Backend
- **Език:** Crystal 1.16+
- **Framework:** Lucky Framework 1.3+
- **Database:** PostgreSQL 14+
- **ORM:** Avram
- **Auth:** Authentic + JWT
- **Email:** Carbon
- **HTTP:** HTTP::Server

### Frontend
- **Език:** Rust 1.75+
- **Framework:** Leptos
- **Bundler:** Trunk
- **HTTP:** gloo-net
- **State:** Signals (Leptos reactive)

### Интеграции
- **OCR/AI:** Mistral AI
- **Email:** SMTP (Carbon)
- **Exchange Rates:** ECB API
- **VAT Validation:** VIES API

---

## Документация

📚 **Пълната документация се намира в [`docs/`](docs/README.md)**

| Документ | Описание |
|----------|----------|
| [Архитектура](docs/architecture.md) | Общ преглед на системата |
| [Backend](docs/backend.md) | Crystal/Lucky Framework API |
| [Frontend](docs/frontend.md) | Leptos/Rust WASM |
| [Контролизи](docs/CONTROLISY.md) | Интеграция с Контролизи |
| [SAF-T BG](docs/SAFT-BG.md) | Експорт за НАП |
| [Mistral OCR](docs/MISTRAL_OCR_INTEGRATION.md) | AI разпознаване на фактури |
| [Банкови транзакции](docs/bank-transactions.md) | Управление на банкови операции |

---

## Папка old/

Папката `old/` съдържа старата версия на приложението и не се използва в текущия проект. Може да бъде използвана само за референция.

---

## Поддръжка

За въпроси и проблеми, моля използвайте GitHub issues.

---

**Последна актуализация:** 30 Януари 2026
# baraba-leptos
