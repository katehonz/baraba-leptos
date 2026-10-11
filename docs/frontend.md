# Frontend

Rust, Leptos 0.8, Trunk, gloo-net. Продукционният образ е `docker/frontend.Dockerfile`.

`AppRouter` ползва плоски пътища.

Публични: `/`, `/login`, `/register`, `/forgot-password`, `/reset-password`, `/verify-email`.

Работни: `/dashboard`, `/journal-entries`, `/invoices`, `/invoices/:id/print`, `/warehouse`, `/warehouse/scan`, `/accounts`, `/counterparts`, `/products`, `/settings`, `/system-settings`, `/controlisy-import`, `/vat-returns`, `/fixed-assets`, `/users`, `/companies`, `/currencies`, `/saft-export`, `/saft-movement-mappings`, `/opening-balances`, `/accounting-periods`, `/reports`, `/scanned-invoices`, `/bank-accounts`, `/bank-transactions`, `/bank-matching-rules`, `/profile`, `/admin`, `/roles`, `/audit-logs`, `/dividends`.

Вход с включена 2FA остава на `/login`: първо парола, после код.

## Токен и файлове

`get_token` чете `auth_token` от `localStorage`. `api_get`, `api_post`, `api_put` и `api_delete` слагат `Authorization: Bearer` и `X-Company-ID`.

Файлове, които не са UTF-8, минават през `api_get_bytes` и `download_bytes`. Дневниците за НАП остават Windows-1251. Адрес, отворен в нов раздел, няма токена и сървърът връща, че няма вход.

SAF-T и UBL ползват същия токен и свалят UTF-8 XML. Нито един от трите експорта не пита за паролата на архива.
