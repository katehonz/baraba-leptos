# Frontend TODO (Karax Nim)

## Текущо състояние
- [x] Базова структура с Karax
- [x] App state management
- [x] Login страница
- [x] Страници: Invoices, Accounts, Products, Counterparts, Settings
- [x] Sidebar navigation

---

## Приоритетни задачи

### 1. API интеграция
- [ ] HTTP модул за fetch заявки
- [ ] Auth модул за управление на токени
- [ ] Error handling и retry логика
- [ ] Response parsing и validation

### 2. Dashboard страница
- [ ] API ендпоинт за статистики
- [ ] Зареждане на реални данни от backend
- [ ] Widgets: общо фактури, приходи/разходи, последни транзакции
- [ ] Графики (може с Chart.js или nim chart библиотеки)

### 3. Companies страница
- [ ] Company модел и типове
- [ ] CRUD операции през API
- [ ] Форма за добавяне/редактиране на фирма
- [ ] Избор на активна фирма

### 4. Counterparts (Контрагенти)
- [ ] Counterpart модел
- [ ] Списък контрагенти от API
- [ ] Филтриране: клиенти/доставчици
- [ ] Форма за добавяне/редактиране

### 5. Bank Accounts (Банкови сметки)
- [ ] BankAccount модел
- [ ] Показване на баланс
- [ ] Списък транзакции за сметка

---

## Компоненти за създаване

### UI компоненти
- [ ] `components/button.nim` - бутон с варианти (primary, secondary, danger)
- [ ] `components/input.nim` - input поле с label и error
- [ ] `components/select.nim` - dropdown
- [ ] `components/table.nim` - таблица с пагинация
- [ ] `components/modal.nim` - модален прозорец
- [ ] `components/card.nim` - карта за dashboard widgets
- [ ] `components/toast.nim` - нотификации
- [ ] `components/loading.nim` - loading spinner
- [ ] `components/pagination.nim` - пагинация

### Layout компоненти
- [ ] `components/sidebar.nim` - странично меню (вече имплементирано в app.nim)
- [ ] `components/header.nim` - горно меню с user info

---

## API Endpoints (вече готови в backend)

```
POST   /api/sign_ups
POST   /api/sign_ins
GET    /api/me
GET    /api/health
GET    /api/companies
POST   /api/companies
GET    /api/companies/:id
PUT    /api/companies/:id
DELETE /api/companies/:id
GET    /api/companies/:id/counterparts
POST   /api/companies/:id/counterparts
GET    /api/companies/:id/accounts
GET    /api/companies/:id/invoices
GET    /api/companies/:id/bank_accounts
```

---

## Примерен код (Nim)

### Store шаблон
```nim
import std/[asyncjs, json, dom, jsffi]

type
  Item = object
    id: int
    name: string

  ExampleStore = ref object
    items: seq[Item]
    loading: bool
    error: string

proc fetchAll(store: ExampleStore, companyId: int): Future[void] {.async.} =
  store.loading = true
  try:
    let response = await fetch(API_BASE & "/api/companies/" & $companyId & "/examples")
    let text = await response.text()
    let data = parseJson($text)
    store.items = data["items"].to(seq[Item])
  except:
    store.error = "Failed to fetch"
  store.loading = false

proc create(store: ExampleStore, companyId: int, data: JsonNode): Future[bool] {.async.} =
  try:
    let body = %*{"item": data}
    let response = await fetch(
      API_BASE & "/api/companies/" & $companyId & "/examples",
      RequestOptions(
        method: httpPost,
        headers: newHttpHeaders([("Content-Type", "application/json")]),
        body: $body
      )
    )
    if response.status == 200:
      await store.fetchAll(companyId)
      return true
    return false
  except:
    store.error = "Failed to create"
    return false
```

### Page компонент шаблон
```nim
import karax/prelude

proc renderExample(): VNode =
  buildHtml(tdiv(class="page")):
    tdiv(class="page-header"):
      h1: text "Examples"
      button:
        text "Add New"

    if state.loading:
      renderLoading()
    else:
      renderTable(state.items)

proc renderTable(items: seq[Item]): VNode =
  buildHtml(tdiv):
    table:
      thead:
        tr:
          th: text "ID"
          th: text "Name"
      tbody:
        for item in items:
          tr:
            td: text $item.id
            td: text item.name
```

---

## Структура на проект (Karax)

```
frontend/
├── src/
│   ├── app.nim                 # Основно приложение
│   ├── frontend/
│   │   ├── submodule.nim       # Frontend модул
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
│       ├── test1.nim
│       └── config.nims
├── index.html
└── frontend.nimble
```

---

## Стилове

Използвай inline CSS в Karax компонентите:
- TailwindCSS класове са опция чрез CDN
- Или създай отделен `styles/` папка с CSS файлове

---

## Следващи стъпки (по ред)

1. **API модул** - основа за всички заявки към backend
2. **Auth модул** - управление на токени и login/logout
3. **Login страница интеграция** - реален login с API
4. **Dashboard с реални данни** - зареждане на статистики
5. **Companies CRUD** - пълна функционалност за фирми
6. **Counterparts CRUD** - управление на контрагенти
7. **Invoices CRUD** - фактури с линии
8. **Accounts CRUD** - сметкоплан
