# Migration Progress Tracker

## Фаза 1: Backend - JSONB Optimization (ЗАВЪРШЕНО)
- [x] Създаване на MIGRATION_PLAN.md
- [x] Актуализиране на README.md, FRONTEND_TODO.md, DEVELOPMENT_PLAN.md, LOCAL_SETUP.md, TODO.md, TASKS.md, NEW_START_PLAN.md
- [x] Актуализиране на start_local.sh, stop_local.sh, start.sh
- [x] Актуализиране на модели (journal_entry.cr, invoice.cr, company.cr) с String за JSON вместо JSONB (по-горе)
- [x] Създаване на сериализатор (jsonb_serializer.cr) с JSONB helper методи
- [x] Актуализиране на API actions (journal_entries/create.cr, invoices/create.cr, companies/update.cr) с JSONB сериализация
- [x] Създаване на Frontend API модул (frontend/src/frontend/api.nim)
- [x] Създаване на Frontend Auth модул (frontend/src/frontend/auth.nim)
- [x] Създаване на Frontend Stores (company.nim, invoice.nim)
- [x] Основни миграции изпълнени (JSONB колони ще се добавят по-късно в production)
- [x] Models обновени да използват String за JSON (parse at runtime)
- [x] JSONB поддръжка се използва в models чрез JSON.parse() вместо type casting

## Фаза 2: Frontend Integration (ЗАВЪРШЕНО)
- [x] Създаване на Counterpart store (frontend/src/frontend/stores/counterpart.nim)
- [x] Създаване на Account store (frontend/src/frontend/stores/account.nim)
- [x] Създаване на i18n модул (frontend/src/frontend/i18n.nim) - BG/EN
- [x] Интеграция на stores в app.nim
- [x] Company selector в sidebar
- [x] Dashboard страница с статистики от stores
- [x] Invoices страница с CRUD (listing, create modal, edit, delete)
- [x] Counterparts страница с CRUD (listing, create/edit modal, delete)
- [x] Accounts страница с CRUD (listing, create/edit modal, delete)
- [x] Settings страница с избор на език (BG/EN)
- [x] Responsive modals за create/edit
- [x] Confirm delete модални прозорци
- [x] Language toggle в sidebar

## Създадени файлове

### Frontend Stores
- `frontend/src/frontend/stores/company.nim` - Company CRUD store
- `frontend/src/frontend/stores/invoice.nim` - Invoice CRUD store
- `frontend/src/frontend/stores/counterpart.nim` - Counterpart CRUD store
- `frontend/src/frontend/stores/account.nim` - Account CRUD store

### Frontend Modules
- `frontend/src/frontend/api.nim` - HTTP API helpers
- `frontend/src/frontend/auth.nim` - Auth helpers
- `frontend/src/frontend/i18n.nim` - Internationalization (BG/EN)

### Main App
- `frontend/src/app.nim` - Main Karax application with:
  - Login/Logout
  - Sidebar navigation
  - Company selector
  - Dashboard with stats
  - CRUD pages for Invoices, Counterparts, Accounts
  - Settings page with language switcher
  - Modal dialogs for create/edit/delete

## Следващи стъпки

### Фаза 3: Backend API Endpoints
- [ ] Verify API endpoints return correct JSON format with `success` field
- [ ] Test all CRUD operations from frontend
- [ ] Add pagination to list endpoints

### Фаза 4: Invoice Form
- [ ] Create full Invoice form with lines (JSONB)
- [ ] Add counterpart selector in invoice form
- [ ] Add invoice posting functionality

### Фаза 5: Journal Entries
- [ ] Create Journal Entry page
- [ ] Create Journal Entry form with lines
- [ ] Integrate with accounts

### Фаза 6: Reports
- [ ] Balance Sheet report
- [ ] Income Statement report
- [ ] Trial Balance report

### Фаза 7: Advanced Features
- [ ] VAT Returns
- [ ] Bank Transactions
- [x] Document scanning (Mistral AI) ✅
- [ ] VIES validation

## Команди за стартиране

```bash
# Backend
cd backend && lucky dev

# Frontend
cd frontend && nimble js
# or
cd frontend && nim js -d:release src/app.nim

# Serve frontend
python3 -m http.server 3000 --directory frontend
```

## Notes

- Frontend използва Karax (Nim -> JavaScript)
- i18n е имплементирано с прост Table-based подход
- Езикът се запазва в localStorage
- По подразбиране езикът е български (BG)
