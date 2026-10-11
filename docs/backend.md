# Backend

Crystal 1.17.0, Lucky, Avram, PostgreSQL 16. Паролата минава през Authentic. Сесията е JWT в `Authorization: Bearer`. 2FA е TOTP по желание на потребителя.

Продукционният `.env` е в корена на хранилището. Образът компилира `src/start_server.cr` с `crystal build --release`. При старт сървърът сам пуска чакащите миграции.

```bash
docker compose -f docker-compose.prod.yml build backend
docker compose -f docker-compose.prod.yml up -d backend
```

`ApiAction` включва `RequireAuthToken` и `RequireCompanyAccess`. Публичните действия включват `SkipRequireAuthToken`.

## Вход

- POST `/api/auth/signin` — парола; при включена 2FA връща `challenge_token`, не сесия
- POST `/api/auth/signin/totp` — код от приложението или код за възстановяване
- POST `/api/auth/two_factor/setup`, `/confirm`, `/disable`, `/recovery_codes`
- POST `/api/auth/signup` и `/api/auth/register`
- GET `/api/auth/me`
- GET `/api/auth/verify_email`
- POST `/api/auth/forgot_password`
- POST `/api/auth/reset_password`

## Фирма

Данните на фирмата са под `/api/companies/:company_id/`.

ДДС периоди:

- GET и POST `/vat_returns`
- GET и PUT `/vat_returns/:id`
- POST `/vat_returns/:id/calculate`
- POST `/vat_returns/:id/import_from_journal`
- GET `/vat_returns/:id/entries`
- GET `/vat_returns/:id/export/deklar|pokupki|prodajbi` — файлове за НАП, Windows-1251. Виж [vat-nap-export.md](vat-nap-export.md)

UBL: GET `/invoices/:invoice_id/ubl`. Иска право `INVOICE_READ`. UTF-8 XML, без парола за архив.

## Общи адреси

- GET `/api/saft/validate`
- GET `/api/saft/export` — UTF-8 XML с обикновен токен
- GET `/api/health`

Архив, само супер администратор:

- POST `/api/backup/create` — иска парола
- GET `/api/backup/list`
- POST `/api/backup/restore` — иска парола
- POST `/api/backup/delete/:filename` — иска парола
- GET и POST `/api/backup/settings`
- POST `/api/backup/test`

ДДС, SAF-T и UBL не минават през тази парола.
