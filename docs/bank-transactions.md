# Банкови транзакции — документация

## Общ преглед

Модулът за банкови транзакции позволява:
1. **Дефиниране на банкови сметки** с обвързана счетоводна (GL) и буферна сметка
2. **Импорт на банкови извлечения** от различни формати (OBB, ISO 20022, Postbank, MT940, CSV)
3. **Автоматично осчетоводяване** при импорт (банка + буферна сметка)
4. **Обработка на транзакции** — замяна на буферната сметка с реалната кореспондираща сметка
5. **Ръчно осчетоводяване** на транзакции без буферна сметка

## Концепция: Буферна сметка

При импорт на банково извлечение банковата сметка (напр. 503) е известна, но кореспондиращата сметка (напр. 602 Разходи за наем) не е. Затова се използва **буферна сметка** (напр. 499 Други кредитори) като временна "другата страна" на записа.

### Пример — депозит (приход):
```
Дт 503 Разплащателна сметка    1000.00
Кт 499 Буферна сметка           1000.00
```

### Пример — теглене (разход):
```
Дт 499 Буферна сметка            500.00
Кт 503 Разплащателна сметка      500.00
```

След обработка буферната сметка се заменя с реалната:
```
Дт 602 Разходи за наем           500.00
Кт 503 Разплащателна сметка      500.00
```

## Статуси на транзакции

| Статус | Описание | Цвят |
|--------|----------|------|
| **Неосчетоводено** | Няма счетоводен запис (journal entry) | Сиво |
| **За обработка** | Има запис, но с буферна сметка | Оранжево |
| **Обработено** | Записът е с реална кореспондираща сметка | Зелено |

Определяне на статуса:
- `is_booked = false` → Неосчетоводено
- `is_booked = true` и `is_allocated = false` → За обработка (journal entry съдържа буферна сметка)
- `is_booked = true` и `is_allocated = true` → Обработено (journal entry НЕ съдържа буферна сметка)

---

## Backend API

Всички endpoints са под `/api/companies/:company_id/`.

### Банкови сметки

#### `GET /bank_accounts`

Връща списък с банкови сметки.

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": 1,
      "name": "ОББ Основна",
      "bank_name": "ОББ",
      "iban": "BG19UBBS80021016251650",
      "bic": "UBBSBGSF",
      "currency": "BGN",
      "gl_account_id": 200,
      "gl_account_code": "503",
      "gl_account_name": "Разплащателна сметки в левове",
      "buffer_account_id": 196,
      "buffer_account_code": "499",
      "buffer_account_name": "Други кредитори",
      "balance": null,
      "transaction_count": 26
    }
  ]
}
```

#### `POST /bank_accounts`

Създава банкова сметка.

**Params (form-encoded, nested `bank_account:`):**
| Поле | Тип | Задължително | Описание |
|------|-----|-------------|----------|
| `name` | String | Да | Име на сметката |
| `bank_name` | String | Не | Име на банката |
| `iban` | String | Не | IBAN (използва се за auto-detect при импорт) |
| `bic` | String | Не | BIC/SWIFT код |
| `currency` | String | Да | Валута (BGN, EUR, USD) |
| `gl_account_id` | Int64 | Не | Счетоводна сметка (клас 5) |
| `buffer_account_id` | Int64 | Не | Буферна сметка (напр. 499) |

> **Важно:** За автоматично осчетоводяване при импорт, и двете (`gl_account_id` и `buffer_account_id`) трябва да са зададени.

---

### Банкови транзакции

#### `GET /bank_transactions`

Връща списък с транзакции.

**Query параметри:**
| Параметър | Описание |
|-----------|----------|
| `bank_account_id` | Филтър по банкова сметка |
| `status` | `booked` / `unbooked` |
| `date_from` | От дата (YYYY-MM-DD) |
| `date_to` | До дата (YYYY-MM-DD) |
| `limit` | Макс. брой (1-1000, default 500) |

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": 1,
      "bank_account_id": 1,
      "bank_account_name": "ОББ Основна",
      "date": "2025-02-03T00:00:00Z",
      "amount": -13.90,
      "currency": "BGN",
      "amount_base": null,
      "description": "МЕС. ТАКСА ПАКЕТ",
      "contra_account": null,
      "contra_name": "ОББ",
      "reference": "REF123",
      "journal_entry_id": 239,
      "is_booked": true,
      "is_allocated": false,
      "journal_lines": [
        {"account_id": 196, "debit": 13.9, "credit": 0.0, "description": "Контрагент"},
        {"account_id": 200, "debit": 0.0, "credit": 13.9, "description": "МЕС. ТАКСА ПАКЕТ"}
      ]
    }
  ]
}
```

**Филтриране по allocation status:**

Frontend-ът използва client-side филтриране за `unallocated` и `allocated`:
- `status=unallocated` → заявка със `status=booked`, после `list.retain(|tx| tx.is_booked && !tx.is_allocated)`
- `status=allocated` → заявка със `status=booked`, после `list.retain(|tx| tx.is_booked && tx.is_allocated)`

---

#### `POST /bank_transactions/import`

Импорт на банково извлечение.

**Params (form-encoded):**
| Поле | Тип | Задължително | Описание |
|------|-----|-------------|----------|
| `file_content` | String | Да | Base64-encoded съдържание на файла |
| `bank_account_id` | Int64 | Не | Банкова сметка (auto-detect по IBAN ако не е зададено) |

**Логика:**
1. Декодира base64 и парсва файла (`BankFileParser.parse`)
2. Ако `bank_account_id` не е зададен, търси по IBAN
3. За всяка транзакция проверява за дубликат по `reference`
4. Създава `BankTransaction` запис
5. **Ако банковата сметка има GL + buffer account** → автоматично създава `JournalEntry`:
   - Депозит (amount > 0): Дт GL, Кт Buffer
   - Теглене (amount < 0): Дт Buffer, Кт GL
6. Свързва транзакцията с journal entry

**Response:**
```json
{
  "success": true,
  "data": {
    "bank_format": "OBB",
    "account_iban": "BG19UBBS80021016251650",
    "account_currency": "BGN",
    "period_from": "2025-02-01T00:00:00Z",
    "period_to": "2025-02-28T00:00:00Z",
    "opening_balance": 980.86,
    "closing_balance": 3752.57,
    "total_count": 29,
    "new_count": 26,
    "duplicate_count": 3,
    "imported_count": 26,
    "journal_count": 26,
    "auto_journal": true,
    "bank_account_id": 1,
    "bank_account_name": "ОББ Основна"
  }
}
```

---

#### `POST /bank_transactions/:bank_transaction_id/reallocate`

Преразпределя буферната сметка към реална сметка.

**Params (form-encoded):**
| Поле | Тип | Задължително | Описание |
|------|-----|-------------|----------|
| `account_id` | Int64 | Да | Новата сметка (замества буферната) |

**Логика:**
1. Намира транзакцията и нейния journal entry
2. Парсва JSON `lines` от journal entry
3. Намира реда с `account_id == buffer_account_id`
4. Заменя `account_id` с новата сметка
5. Записва обратно

**Response:**
```json
{
  "success": true,
  "data": {
    "journal_entry_id": 239,
    "new_account_id": 231,
    "new_account_code": "60202",
    "new_account_name": "Разходи за наем, включително оперативен лизинг",
    "message": "Сметката е преразпределена от буфер към 60202 Разходи за наем"
  }
}
```

**Грешки:**
- 400: Транзакцията няма счетоводен запис
- 400: Моля изберете сметка
- 400: Банковата сметка няма буферна сметка
- 400: Не е намерен ред с буферна сметка

---

#### `POST /bank_transactions/:bank_transaction_id/book`

Ръчно осчетоводяване (за транзакции без автоматичен journal entry).

**Params:** `contra_account_id` — кореспондиращата сметка.

---

## Поддържани формати за импорт

Парсерът (`BankFileParser`) автоматично разпознава формата.

| Формат | Банка | Тип файл | Разпознаване |
|--------|-------|----------|-------------|
| **OBB XML** | Обединена Българска Банка | XML | `<AccountMovements` root tag |
| **ISO 20022 camt.053** | Paysera, Revolut, Wise | XML | `<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053..."` |
| **Postbank XML** | Пощенска банка | XML | `<CustomerReport` root tag |
| **MT940/SWIFT** | Unicredit | TXT | Започва с `:20:` или `{1:` |
| **CSV** | CCBank | CSV | Разделител `;`, header row |

### Полета от парсера

Всеки формат връща унифицирана структура:

```crystal
class BankFileParser::Result
  property bank_format : String?
  property account_iban : String?
  property account_currency : String?
  property period_from : Time?
  property period_to : Time?
  property opening_balance : Float64?
  property closing_balance : Float64?
  property transactions : Array(Transaction)
end

class BankFileParser::Transaction
  property date : Time
  property amount : Float64         # положително = приход, отрицателно = разход
  property currency : String
  property description : String?
  property contra_account : String?  # IBAN/сметка на контрагента
  property contra_name : String?     # Име на контрагента
  property reference : String?       # Уникален референтен номер
end
```

---

## Frontend

### Страници

#### `/bank-accounts` — Банкови сметки

Таблица с всички банкови сметки и модал за създаване/редактиране.

**Полета в модала:**
- Име, Банка, IBAN, BIC, Валута
- **Счетоводна сметка (GL)** — dropdown с филтър по клас 5 (Парични средства)
- **Буферна сметка** — dropdown с всички сметки (напр. 499 Други кредитори)

**Таблица показва:** Име, IBAN, Валута, Счетоводна сметка, Буферна сметка, Салдо, Транзакции

#### `/bank-transactions` — Банкови транзакции

Два режима: **Списък** и **Импорт**.

##### Списък (step 0)

**Филтри:**
- Банкова сметка (dropdown)
- Статус: Всички / Неосчетоводени / За обработка / Обработени / Всички осчетоводени
- От дата / До дата

**Таблица:**
- Дата, Сметка, Описание, Контрагент, Сума (цветно: зелено приход, червено разход)
- Статус badge (сиво/оранжево/зелено)
- Действия:
  - Неосчетоводено: [Осчетоводи] [Изтрий]
  - За обработка: [Обработи] [JE #xxx]
  - Обработено: [JE #xxx]

Транзакциите "За обработка" са с amber/жълт фон.

##### Импорт wizard (steps 1-3)

**Стъпка 1 — Качване:**
- Drag & drop зона или бутон за избор на файл
- Поддържани формати: OBB XML, ISO 20022, Postbank XML, MT940/SWIFT, CSV
- Бутон "Преглед" → preview endpoint

**Стъпка 2 — Преглед:**
- Информация: формат, IBAN, период, валута
- Статистика: нови / дубликати / общо
- Избор на банкова сметка (auto-detect по IBAN)
- Таблица с preview на транзакции (дубликатите — жълт фон)
- Бутон "Импортирай X нови транзакции"

**Стъпка 3 — Готово:**
- Брой импортирани транзакции
- Ако `auto_journal = true`: "Автоматично създадени X счетоводни записа с буферна сметка"
- Бутон **"Обработи транзакциите"** → превключва към списък с филтър "За обработка"
- Ако няма буферна сметка: подсказка да се настрои в Банкови сметки

##### Модал за осчетоводяване (Booking modal)

За транзакции без journal entry. Показва:
- Информация за транзакцията (дата, описание, контрагент, сума)
- Dropdown за кореспондираща сметка
- Предварителен преглед на записа (Дт/Кт)

##### Модал за обработка (Reallocate modal)

За транзакции с буферна сметка. Показва:
- Информация за транзакцията
- **Текущ счетоводен запис** с два реда:
  - Банков ред (син фон, "банка") — read-only
  - Буферен ред (amber фон, "буфер") — за замяна
- Dropdown "Заменете буферната сметка с:"
- Бутон "Преразпредели"

---

## Backend файлове

| Файл | Описание |
|------|----------|
| `models/bank_account.cr` | Модел — `belongs_to gl_account : Account?`, `belongs_to buffer_account : Account?` |
| `models/bank_transaction.cr` | Модел — `belongs_to bank_account`, `belongs_to journal_entry?` |
| `operations/save_bank_account.cr` | `permit_columns` включва `buffer_account_id` |
| `actions/api/bank_accounts/index.cr` | Връща GL + buffer account info |
| `actions/api/bank_accounts/show.cr` | Връща GL + buffer account info |
| `actions/api/bank_accounts/create.cr` | Създаване на банкова сметка |
| `actions/api/bank_transactions/index.cr` | Списък с `is_allocated`, `journal_lines` |
| `actions/api/bank_transactions/import.cr` | Импорт + auto-journal |
| `actions/api/bank_transactions/reallocate.cr` | Преразпределяне на буферна сметка |
| `services/bank_file_parser.cr` | Парсер за банкови файлове |
| `db/migrations/20260214000002_add_buffer_account_to_bank_accounts.cr` | Миграция за `buffer_account_id` |

## Frontend файлове

| Файл | Описание |
|------|----------|
| `leptos/src/pages/bank_accounts.rs` | Банкови сметки — CRUD + buffer account dropdown |
| `leptos/src/pages/bank_transactions.rs` | Транзакции — списък, импорт wizard, booking/reallocate модали |

---

## Типичен workflow

1. **Настройка:** Създайте банкова сметка с GL сметка (503) и буферна сметка (499)
2. **Импорт:** Качете банково извлечение (XML/CSV/MT940)
3. **Преглед:** Проверете броя нови транзакции и дубликати
4. **Импортиране:** Натиснете "Импортирай" → автоматично се създават записи с буферна сметка
5. **Обработка:** Натиснете "Обработи транзакциите" → отваря се списъкът "За обработка"
6. **Преразпределяне:** За всяка транзакция натиснете "Обработи" → изберете реалната сметка → "Преразпредели"
7. **Готово:** Транзакцията преминава в статус "Обработено"

### Без буферна сметка

Ако банковата сметка няма `buffer_account_id`:
- Транзакциите се импортират без journal entry (статус "Неосчетоводено")
- За осчетоводяване натиснете "Осчетоводи" → изберете кореспондираща сметка

---

**Последна актуализация:** 14 Февруари 2026
