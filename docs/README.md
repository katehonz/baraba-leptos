# Документация

Последна сверка с кода: 2026-10-11.

Производството е https://baraba.org. Образите се строият от `docker-compose.prod.yml`: Crystal 1.17.0, PostgreSQL 16, Leptos WASM през Trunk. Миграциите се пускат при старт на `baraba-backend`.

## Какво да се чете

| Файл | За какво е |
|------|------------|
| [architecture.md](architecture.md) | Стек, достъп, 2FA и разликата между файлове за НАП и архив |
| [vat-nap-export.md](vat-nap-export.md) | `DEKLAR.TXT`, `POKUPKI.TXT`, `PRODAGBI.TXT` |
| [SAFT-BG.md](SAFT-BG.md) | SAF-T BG v1.0.1 |
| [backend.md](backend.md) | Вход, ДДС, SAF-T, UBL и архив |
| [frontend.md](frontend.md) | Страници и как клиентът праща токена |
| [bank-transactions.md](bank-transactions.md) | Банкови движения |
| [scanned-invoice-payments.md](scanned-invoice-payments.md) | Плащане от сканирана фактура |
| [warehouse-invoicing-design.md](warehouse-invoicing-design.md) | Складови документи |
| [fs-nss1-design.md](fs-nss1-design.md) | Отчети по НСС 1 |
| [MISTRAL_OCR_INTEGRATION.md](MISTRAL_OCR_INTEGRATION.md) | OCR |

Фирмените данни са под `/api/companies/:company_id/...`. Заявката иска `Authorization: Bearer` и потребител, който е супер администратор, собственик или има роля във фирмата. Токенът стои в `localStorage` (`auth_token`). Директно отваряне на адрес в браузъра няма този заглавен ред и връща `{"error":"Not authenticated"}`.
