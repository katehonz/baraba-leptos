# SAF-T BG - Стандартен одиторски файл за България

## Статус: В разработка (основа готова, задължителен от 2028г. за средни/малки фирми)

Базиран на схема `BG_SAFT_Schema_V_1.0.1.xsd`. Референтни файлове в `SAFT_BG/`.

---

## Какво е направено

### Backend - XML Генератор (`backend/src/services/saft_exporter.cr`)

| Секция | Статус | Описание |
|--------|--------|----------|
| Header | Готово | Версия, фирма, адрес, ДДС рег., банкови сметки, собственост |
| MasterFiles / GeneralLedgerAccounts | Готово | Сметкоплан с SAF-T mapping, начални/крайни салда |
| MasterFiles / Customers | Готово | Клиенти с SAF-T ID, адрес, ДДС, свързани лица |
| MasterFiles / Suppliers | Готово | Доставчици (аналогично на клиенти) |
| MasterFiles / TaxTable | Готово | ДДС ставки: 20%, 9%, 0%, освободени |
| MasterFiles / UOMTable | Готово | Мерни единици: бр, кг, л, м, кв.м, час |
| MasterFiles / Products | Готово | Продукти с код, група, мерна единица, метод на оценка |
| MasterFiles / PhysicalStock | Готово | Физически наличности (само за OnDemand отчет) |
| MasterFiles / Assets | Готово | ДМА с SAP и DAP амортизационни планове (само за Annual отчет) |
| GeneralLedgerEntries | Готово | Счетоводни записи с дебит/кредит, валута, данъчна информация |
| SourceDocuments / SalesInvoices | Готово | Фактури за продажба с редове, ДДС, тотали |
| SourceDocuments / PurchaseInvoices | **Празно** | Stub - връща 0 записа |
| SourceDocuments / Payments | Готово | Плащания с редове, дебит/кредит индикатор |
| SourceDocuments / MovementOfGoods | Готово | Движение на стоки (само OnDemand) |
| SourceDocuments / AssetTransactions | Готово | Операции с ДМА (само Annual) |

### Три типа отчети
- **Monthly** - месечен (основен)
- **OnDemand** - при поискване (+ PhysicalStock, MovementOfGoods)
- **Annual** - годишен (+ Assets, AssetTransactions)

### Backend - API Endpoints

| Endpoint | Метод | Описание |
|----------|-------|----------|
| `/api/saft/validate` | GET | Валидация преди експорт |
| `/api/saft/export` | GET | Генериране и сваляне на XML |
| `/api/companies/:id/saft/movement_mappings` | CRUD | Mapping на стокови кореспонденции |
| `/api/companies/:id/saft/asset_mappings` | CRUD | Mapping на ДМА кореспонденции |
| `/api/companies/:id/saft/cash_mappings` | CRUD | Mapping на парични кореспонденции |
| `/api/saft_accounts` | GET | НАП номенклатура на сметки |
| `/api/saft_accounts/load` | POST | Зареждане от CSV |

### Backend - Автоматични генерации

- **SAF-T ID за контрагенти** - генерира се при създаване:
  - `10` + ЕИК (български фирми)
  - `11` + CC + VAT (EU фирми)
  - `12` + CC + ID (извън EU)
  - `13` + ЕГН (физически лица)
  - `15` + timestamp (fallback)
- **TransactionID за статии** - `JE-{timestamp}-{random}` при създаване

### Backend - Модели и номенклатури

- `SaftAccount` - НАП стандартен сметкоплан (от `data/NRA_Nom_Accounts.csv`)
- `SaftMovementAccountMapping` - кореспонденции за стокови движения (wildcard patterns)
- `SaftAssetAccountMapping` - кореспонденции за ДМА операции
- `SaftCashAccountMapping` - кореспонденции за парични движения
- `SaftInvoiceType`, `SaftTaxType`, `SaftPaymentMethod`, `SaftRegion`, `SaftProductType`, `SaftUnitOfMeasure`, `SaftIbanFormat`, `SaftTaxRegime`, `SaftCashMovementType`, `SaftAssetMovementType`, `SaftStockMovementType`

### Frontend

| Страница | Път | Описание |
|----------|-----|----------|
| SAF-T Експорт | `/saft-export` | Избор на период, валидация, сваляне на XML |
| SAF-T Кореспонденции | `/saft-movement-mappings` | 3 таба: Стоки, ДМА, Парични. CRUD за mapping-и |
| Сметкоплан | `/accounts` | SAF-T mapping dropdown + "Отчитане на артикули" чекбокс в модалите |

### Валидация преди експорт

Проверява и показва:
- **Грешки** (блокиращи): ЕИК не е попълнен, небалансирани записи
- **Предупреждения**: липсващи SAF-T ID, липсващи mapping-и, липсващи TransactionID
- **Статистика**: брой статии, фактури, контрагенти, сметки за периода

---

## Какво остава да се направи

### Приоритет 1 - Преди пускане

- [ ] **Покупни фактури** - `build_purchase_invoices` е празен stub. Трябва да се свърже с реалните покупни документи (от Controlisy import или ръчно въведени)
- [ ] **XSD валидация** - генерираният XML да се валидира срещу `BG_SAFT_Schema_V_1.0.1.xsd` и да се поправят несъответствията
- [ ] **Тестване с реални данни** - натрупване на достатъчно данни за пълен тест (очакване: 2027-2028)

### Приоритет 2 - Подобрения

- [ ] **OwnershipStructure UI** - страница/форма в настройки на фирмата за попълване на:
  - Бенефициери (BeneficialOwner)
  - Крайни собственици (UltimateParent)
  - `is_part_of_group` флаг
- [x] **SAF-T Account mapping UI** - searchable dropdown в модала за сметки (Нова сметка / Редактиране) за свързване с НАП стандартни сметки
- [ ] **Множество ДДС ставки** - сега е hardcoded `100211` (20%) на много места. Трябва динамично определяне по данните
- [ ] **Мултивалутност** - конвертиране при различна от BGN валута (ExchangeRate)
- [ ] **Batch export** - експорт за няколко месеца наведнъж

### Приоритет 3 - Бъдещо

- [ ] **Автоматичен SAF-T account mapping** - по код на сметката да предлага НАП стандартна сметка
- [ ] **Разширени номенклатури** - `saft_extended_nomenclatures` таблицата съществува но не се ползва активно
- [ ] **Подписване** - електронен подпис на XML файла (ако НАП го изиска)
- [ ] **Директно подаване** - API интеграция с НАП (когато стане достъпно)

---

## Референтни файлове

```
SAFT_BG/
├── BG_SAFT_Schema_V_1.0.1.xsd          # XML Schema
├── SAF-T_BG_Format_Reporting.pdf         # Спецификация (Приложение 3)
├── Structure_Definition_V_1.0.1.xlsx     # Дефиниция на структурата
├── VS_SAMPLE_AuditFile_Monthly_V_1.0.1.xml   # Примерен месечен
├── VS_SAMPLE_AuditFile_Annual_V_1.0.xml       # Примерен годишен
└── VS_SAMPLE_AuditFile_OnDemand_V_1.0.xml     # Примерен при поискване
```

## Ключови файлове

```
backend/src/services/saft_exporter.cr           # Главен XML генератор (925 реда)
backend/src/services/saft_asset_mapper.cr        # Mapper за ДМА кореспонденции
backend/src/services/saft_movement_mapper.cr     # Mapper за стокови кореспонденции
backend/src/actions/api/saft/export.cr           # API endpoint за експорт
backend/src/actions/api/saft/validate.cr         # API endpoint за валидация
backend/src/operations/save_counterpart.cr       # Auto SAF-T ID generation
backend/src/operations/save_journal_entry.cr     # Auto TransactionID generation
leptos/src/pages/saft_export.rs                  # Frontend страница за експорт
leptos/src/pages/saft_movement_mappings.rs       # Frontend за кореспонденции
```
