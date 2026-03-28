# Baraba-2 План за разработка - Чист старт

## Дата: 2025-01-25 -> 2026-01-26
## Статус: ✅ Ново начало
## Архитектура: Crystal/Lucky Backend + Karax Nim Frontend

---

## Решение

Започваме baraba-2 на чисто с:
- ✅ PostgreSQL JSONB колони за сложни връзки (journal_lines, invoice_lines)
- ✅ Чисто нова имплементация на Crystal/Lucky
- ✅ Karax Nim за frontend (компилира се до JavaScript)
- ✅ Без миграции на данни
- ✅ kankrum само като референция за функционалност

---

## Database Schema с JSONB

```sql
-- Companies
CREATE TABLE companies (
  id BIGSERIAL PRIMARY KEY,
  name VARCHAR NOT NULL,
  vat_number VARCHAR,
  address VARCHAR,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

-- Accounts
CREATE TABLE accounts (
  id BIGSERIAL PRIMARY KEY,
  company_id BIGINT NOT NULL,
  code VARCHAR NOT NULL,
  name VARCHAR NOT NULL,
  account_type VARCHAR NOT NULL,
  is_active BOOLEAN DEFAULT true,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

-- Counterparts
CREATE TABLE counterparts (
  id BIGSERIAL PRIMARY KEY,
  company_id BIGINT NOT NULL,
  name VARCHAR NOT NULL,
  vat_number VARCHAR,
  address VARCHAR,
  contact_person VARCHAR,
  email VARCHAR,
  phone VARCHAR,
  counterpart_type VARCHAR NOT NULL,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

-- Journal Entries с JSONB за lines
CREATE TABLE journal_entries (
  id BIGSERIAL PRIMARY KEY,
  company_id BIGINT NOT NULL,
  entry_date TIMESTAMP NOT NULL,
  description VARCHAR NOT NULL,
  reference VARCHAR,
  status VARCHAR NOT NULL,
  lines JSONB DEFAULT '[]'::jsonb,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

-- Invoices с JSONB за lines
CREATE TABLE invoices (
  id BIGSERIAL PRIMARY KEY,
  company_id BIGINT NOT NULL,
  counterpart_id BIGINT NOT NULL,
  invoice_number VARCHAR NOT NULL,
  invoice_date TIMESTAMP NOT NULL,
  due_date TIMESTAMP,
  subtotal NUMERIC NOT NULL,
  vat_amount NUMERIC NOT NULL,
  total_amount NUMERIC NOT NULL,
  currency VARCHAR NOT NULL,
  status VARCHAR NOT NULL,
  notes VARCHAR,
  lines JSONB DEFAULT '[]'::jsonb,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);
```

---

## Стъпки за разработка

### Фаза 1: Database (JSONB)
1. ✅ CreateCompanies миграция
2. ✅ CreateAccounts миграция
3. ✅ CreateCounterparts миграция
4. ✅ CreateJournalEntries миграция (с JSONB lines)
5. ✅ CreateInvoices миграция (с JSONB lines)

### Фаза 2: Models (JSONB)
- [ ] Company модел
- [ ] Account модел
- [ ] Counterpart модел
- [ ] JournalEntry модел с lines: JSONB::Any
- [ ] Invoice модел с lines: JSONB::Any

### Фаза 3: Backend API
- [ ] Companies CRUD endpoints
- [ ] Accounts CRUD endpoints
- [ ] Counterparts CRUD endpoints
- [ ] Journal Entries CRUD endpoints (с JSONB)
- [ ] Invoices CRUD endpoints (с JSONB)

### Фаза 4: Frontend (Karax Nim)
- [ ] HTTP модул за API заявки
- [ ] Auth модул за управление на токени
- [ ] Stores за управление на състоянието (CompanyStore, InvoiceStore, и т.н.)
- [ ] Реален login с API
- [ ] Динамични данни от API
- [ ] CRUD форми за всички ресурси

---

## Предимства на Karax Nim подхода

1. **Компилируем**: Nim се компилира до JavaScript, висока производителност
2. **Type-safe**: Статично типизиран, като TypeScript, но по-бърз
3. **Virtual DOM**: Karax използва виртуален DOM, като React
4. **Кратък синтаксис**: По-малко код от JavaScript/TypeScript
5. **Едни език**: Backend (Crystal) и Frontend (Nim) - и двете статично типизирани

---

## Предимства на JSONB подхода

1. **По-просто**: Една таблица вместо две (entries + lines)
2. **По-бързо**: Един SQL query вместо JOIN
3. **По-гъвкаво**: Естествена релация (1:many в JSON)
4. **По-лесно за backup**: Една таблица вместо две
5. **По-добра производителност**: PostgreSQL е оптимизиран за JSONB

---

## Структура на Frontend (Karax Nim)

```
frontend/
├── src/
│   ├── app.nim                 # Основно приложение
│   ├── frontend/
│   │   ├── api.nim             # API helpers
│   │   ├── auth.nim            # Auth helpers
│   │   ├── stores/
│   │   │   ├── company.nim
│   │   │   ├── dashboard.nim
│   │   │   └── ...
│   │   ├── components/
│   │   │   ├── button.nim
│   │   │   ├── input.nim
│   │   │   └── ...
│   │   └── pages/
│   │       ├── login.nim
│   │       ├── dashboard.nim
│   │       └── ...
│   └── tests/
├── index.html
└── frontend.nimble
```

---

## Следващи стъпки

1. Обновяване на JournalEntry и Invoice моделите с JSONB
2. Създаване на API endpoints за CRUD
3. HTTP модул във frontend за API заявки
4. Auth модул във frontend за управление на токени
5. Stores за управление на състоянието
6. Актуализация на frontend с реални API calls

---

**Статус:** Започваме на чисто! 🚀
