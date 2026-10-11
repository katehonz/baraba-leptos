# Архитектура

Baraba е счетоводно приложение: Crystal API и Leptos SPA.

- Backend: Crystal 1.17.0, Lucky, Avram. Образът е `docker/backend.Dockerfile`.
- Frontend: Rust, Leptos 0.8, Trunk, WASM.
- База: PostgreSQL 16.
- Производство: https://baraba.org от `docker-compose.prod.yml`. Caddy праща `/api` към backend и останалото към frontend.

Фирмените данни са под `/api/companies/:company_id/`.

## Достъп

`POST /api/auth/signin` дава JWT за 7 дни. Клиентът го пази в `localStorage` като `auth_token` и го праща с `Authorization: Bearer`. API действията не ползват бисквитка.

Ако потребителят е включил 2FA, signin връща `requires_2fa` и `challenge_token` за 5 минути (`purpose=2fa`). Този токен не отваря защитени адреси. Сесията идва от `POST /api/auth/signin/totp` с код от приложението или код за възстановяване.

Настройката е от профила: `POST /api/auth/two_factor/setup`, после `confirm`. Изключване и нови кодове искат вече отворена сесия.

`ApiAction` иска токен, освен действия с `SkipRequireAuthToken`. Път под `/api/companies/:company_id/` иска супер администратор, собственик или роля във фирмата.

## Файлове и архив

ДДС файловете за НАП, SAF-T XML и UBL XML се свалят с обикновения токен. Виж [vat-nap-export.md](vat-nap-export.md).

Създаване, възстановяване и изтриване на архив искат супер администратор и паролата на акаунта. Списъкът и S3 настройките също са само за супер администратор.

## Пускане

Чакащите миграции се прилагат при старт на backend. Отделна команда за миграция няма.

```bash
docker compose -f docker-compose.prod.yml build backend frontend
docker compose -f docker-compose.prod.yml up -d backend frontend
```

`crystal build --release` компилира и зависимостите. Контейнерът на базата не се пресъздава.

## Код

- `backend/src/actions` — HTTP действия
- `backend/src/models`, `queries`, `operations`, `services` — данни и правила
- `backend/db/migrations` — схема; пазят се, защото нов празен сървър ги пуска отначало
- `leptos/src/pages` — екрани
- `leptos/src/api.rs` — клиент с Bearer токен
