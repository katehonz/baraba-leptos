# Baraba 2 - План за разработка

## Завършени задачи

### Backend (Crystal/Lucky)
- [x] Базова структура на проекта
- [x] User model и JWT автентикация
- [x] API endpoints за auth (/api/sign_ups, /api/sign_ins, /api/me)
- [x] Error handling

### Frontend (Karax Nim)
- [x] Базова структура с Karax framework
- [x] App state management
- [x] Login страница
- [x] Sidebar navigation
- [x] Страници: Dashboard, Invoices, Accounts, Counterparts, Products, Settings

---

## Предстоящи задачи

### 1. База данни - Models и Migrations
- [ ] Company модел и миграция
- [ ] Account модел и миграция
- [ ] Counterpart модел и миграция
- [ ] Journal Entry модел и миграция
- [ ] Journal Line модел и миграция
- [ ] Invoice модел и миграция
- [ ] Invoice Line модел и миграция
- [ ] Bank Account модел и миграция
- [ ] Product модел и миграция

### 2. База данни - Seed данни
- [ ] Seed данни за референтни таблици:
  - [ ] IsoCountry (държави)
  - [ ] IsoCurrency (валути)
  - [ ] SaftInvoiceType
  - [ ] SaftPaymentMethod
  - [ ] SaftTaxType
  - [ ] CombinedNomenclature (митнически кодове)
  - [ ] VatRate (ДДС ставки за България)

### 3. Backend API - CRUD операции
- [ ] Companies API (index, create, show, update, delete)
- [ ] Accounts API (index, create, show, update, delete)
- [ ] Counterparts API (index, create, show, update, delete)
- [ ] Journal Entries API (index, create, show, update, delete, post)
- [ ] Invoices API (index, create, show, update, delete, pdf)
- [ ] Bank Accounts API
- [ ] Products API

### 4. Frontend - HTTP и Auth модули
- [ ] HTTP модул за API заявки (GET, POST, PUT, DELETE)
- [ ] Auth модул за управление на токени
- [ ] Error handling и retry логика
- [ ] Response parsing и validation

### 5. Frontend - Stores
- [ ] CompanyStore
- [ ] AccountStore
- [ ] CounterpartStore
- [ ] InvoiceStore
- [ ] DashboardStore
- [ ] BankAccountStore
- [ ] ProductStore

### 6. Frontend - Страници с реални данни
- [ ] Login/Register с реален API
- [ ] Dashboard с обобщена информация
- [ ] Companies - управление на фирми
- [ ] Counterparts - контрагенти
- [ ] Bank Accounts - банкови сметки
- [ ] Journal Entries - счетоводни записи
- [ ] Invoices - фактури
- [ ] Products - продукти
- [ ] Settings - настройки

### 7. Frontend функционалност
- [ ] Форми за създаване/редактиране на записи
- [ ] Таблици с пагинация и сортиране
- [ ] Търсене и филтриране
- [ ] Валидация на форми
- [ ] Модални прозорци за потвърждение
- [ ] Toast нотификации
- [ ] Loading състояния
- [ ] Error handling

### 8. Backend подобрения
- [ ] Добавяне на пагинация към Index endpoints
- [ ] Филтриране и сортиране на списъци
- [ ] Валидации в SaveOperations:
  - [ ] ДДС номер формат (BG + 9/10 цифри)
  - [ ] IBAN валидация
  - [ ] Дати (invoice_date <= due_date)
- [ ] Бизнес логика:
  - [ ] Автоматично изчисление на invoice totals
  - [ ] Баланс проверка за journal entries
  - [ ] Амортизация на ДМА
- [ ] VIES валидация на ДДС номера (EU API)
- [ ] Интеграция със Salt Edge за банкови връзки
- [ ] Интеграция с Wise API
- [ ] SAF-T XML експорт за НАП

### 9. Документи и файлове
- [ ] Upload на фактури (PDF/Image)
- [x] OCR обработка с Mistral AI ✅
- [ ] S3 storage за файлове
- [ ] Preview на документи

### 10. Справки
- [ ] Оборотна ведомост
- [ ] Главна книга
- [ ] Дневник на покупките/продажбите
- [ ] ДДС справка-декларация
- [ ] VIES декларация
- [ ] Интрастат декларация
- [ ] SAF-T експорт

### 11. Тестове
- [ ] Unit тестове за модели
- [ ] Integration тестове за API
- [ ] E2E тестове за frontend

### 12. DevOps
- [ ] Docker compose за development
- [ ] Production deployment конфигурация
- [ ] CI/CD pipeline
- [ ] Мониторинг и логове

---

## Технически дълг
- [ ] Добавяне на индекси в базата данни за често използвани колони
- [ ] Оптимизация на N+1 заявки
- [ ] Кеширане на референтни данни
- [ ] Rate limiting за API
- [ ] API версиониране

---

## Приоритети (следващи стъпки)
1. Създай HTTP модул във frontend
2. Създай Auth модул във frontend
3. Компания модел и API
4. Login/Register с реален API
5. Dashboard с реални данни
6. Counterparts CRUD
7. Invoices CRUD
8. Accounts CRUD
