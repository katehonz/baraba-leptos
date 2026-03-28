# Frontend документация

## Технологичен стек

- **Език:** Rust 1.75+
- **Framework:** Leptos (React-like, compiled to WASM)
- **Bundler:** Trunk (WASM bundler)
- **HTTP:** gloo-net
- **State Management:** Leptos Signals (Reactive)

## Стартиране

### Development mode

```bash
cd leptos
trunk serve
```

Frontend ще стартира на `http://localhost:8080`

Тrunk автоматично ще:
- Компилира Rust към WASM
- Следи за промени и rebuild-ва
- Reload-ва браузъра

### Production build

```bash
cd leptos
trunk build --release
```

Резултатът ще бъде в `leptos/dist/`

## Структура на frontend

```
leptos/
├── src/
│   ├── main.rs              # Entry point
│   ├── app.rs               # Главен App компонент
│   ├── router.rs            # Router конфигурация
│   ├── api.rs               # API клиент
│   ├── models.rs            # Frontend модели (типове)
│   ├── context/             # App context
│   ├── constants/           # Константи
│   ├── components/          # Reusable UI компоненти
│   ├── pages/               # Страници/Views (30+ страници)
│   ├── stores/              # State management
│   └── i18n/                # Преводи
├── assets/                  # CSS, картинки
├── style/                   # Глобални стилове
├── index.html               # Entry HTML
├── Cargo.toml               # Rust зависимости
└── Trunk.toml               # Trunk конфигурация
```

## Директория `src/`

### main.rs

Entry point на приложението.

```rust
fn main() {
    // Инициализация на i18n
    // Инициализация на app
    leptos::mount_to_body(App)
}
```

### app.rs

Главен App компонент.

```rust
#[component]
pub fn App() -> impl IntoView {
    let (current_user, set_current_user) = create_signal(None);
    let (theme, set_theme) = create_signal("light");

    view! {
        <Router>
            <Routes>
                <Route path="/" view=Dashboard/>
                <Route path="/login" view=Login/>
                // ... други routes
            </Routes>
        </Router>
    }
}
```

### router.rs

Router конфигурация.

```rust
pub fn router() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                // Auth
                <Route path="/login" view=Login/>
                <Route path="/register" view=Register/>
                <Route path="/forgot-password" view=ForgotPassword/>
                <Route path="/reset-password" view=ResetPassword/>
                <Route path="/verify-email" view=VerifyEmail/>

                // Main app (protected)
                <Route path="/" view=AppLayout>
                    <Route path="" view=Dashboard/>

                    // Core Accounting
                    <Route path="invoices" view=InvoicesIndex/>
                    <Route path="invoices/:id" view=InvoicesShow/>
                    <Route path="invoices/new" view=InvoicesNew/>
                    <Route path="accounts" view=AccountsIndex/>
                    <Route path="accounts/:id" view=AccountsShow/>
                    <Route path="accounts/new" view=AccountsNew/>
                    <Route path="counterparts" view=CounterpartsIndex/>
                    <Route path="counterparts/:id" view=CounterpartsShow/>
                    <Route path="counterparts/new" view=CounterpartsNew/>
                    <Route path="journal-entries" view=JournalEntriesIndex/>
                    <Route path="journal-entries/:id" view=JournalEntriesShow/>
                    <Route path="journal-entries/new" view=JournalEntriesNew/>
                    <Route path="payments" view=PaymentsIndex/>
                    <Route path="accounting-periods" view=AccountingPeriodsIndex/>
                    <Route path="opening-balances" view=OpeningBalancesIndex/>

                    // Assets & Inventory
                    <Route path="products" view=ProductsIndex/>
                    <Route path="products/:id" view=ProductsShow/>
                    <Route path="products/new" view=ProductsNew/>
                    <Route path="fixed-assets" view=FixedAssetsIndex/>
                    <Route path="fixed-assets/:id" view=FixedAssetsShow/>
                    <Route path="fixed-assets/new" view=FixedAssetsNew/>
                    <Route path="warehouse" view=WarehouseIndex/>
                    <Route path="stock-transactions" view=StockTransactionsIndex/>

                    // Financial
                    <Route path="bank-accounts" view=BankAccountsIndex/>
                    <Route path="bank-accounts/:id" view=BankAccountsShow/>
                    <Route path="bank-accounts/new" view=BankAccountsNew/>
                    <Route path="bank-transactions" view=BankTransactionsIndex/>
                    <Route path="currencies" view=CurrenciesIndex/>
                    <Route path="exchange-rates" view=ExchangeRatesIndex/>
                    <Route path="vat-returns" view=VatReturnsIndex/>
                    <Route path="dividends" view=DividendsIndex/>

                    // Integrations
                    <Route path="saft-export" view=SaftExport/>
                    <Route path="saft-movement-mappings" view=SaftMovementMappings/>
                    <Route path="controlisy-import" view=ControlisyImport/>
                    <Route path="scanned-invoices" view=ScannedInvoicesIndex/>
                    <Route path="documents" view=DocumentsIndex/>

                    // Management
                    <Route path="users" view=UsersIndex/>
                    <Route path="users/:id" view=UsersShow/>
                    <Route path="roles" view=RolesIndex/>
                    <Route path="companies" view=CompaniesIndex/>
                    <Route path="reports" view=ReportsIndex/>

                    // Admin
                    <Route path="admin" view=AdminDashboard/>
                    <Route path="system-settings" view=SystemSettings/>

                    // User
                    <Route path="settings" view=Settings/>
                    <Route path="profile" view=Profile/>
                </Route>
            </Routes>
        </Router>
    }
}
```

### api.rs

API клиент за HTTP заявки към backend.

```rust
use gloo_net::http::Request;

pub async fn get(url: &str, token: Option<&str>) -> Result<serde_json::Value, String> {
    let mut request = Request::get(url);
    if let Some(t) = token {
        request = request.header("Authorization", &format!("Bearer {}", t));
    }
    request.send().await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

pub async fn post(url: &str, body: &serde_json::Value, token: Option<&str>) -> Result<serde_json::Value, String> {
    let mut request = Request::post(url)
        .header("Content-Type", "application/json");
    if let Some(t) = token {
        request = request.header("Authorization", &format!("Bearer {}", t));
    }
    request.json(body).unwrap().send().await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

pub async fn put(url: &str, body: &serde_json::Value, token: Option<&str>) -> Result<serde_json::Value, String> {
    // ...
}

pub async fn delete(url: &str, token: Option<&str>) -> Result<serde_json::Value, String> {
    // ...
}

// API endpoints
pub async fn login(email: &str, password: &str) -> Result<LoginResponse, String> {
    let body = json!({
        "user": {
            "email": email,
            "password": password
        }
    });
    post("/api/sign_ins", &body, None).await
        .and_then(|res| serde_json::from_value::<LoginResponse>(res).map_err(|e| e.to_string()))
}

pub async fn get_invoices(token: &str) -> Result<Vec<Invoice>, String> {
    get("/api/invoices", Some(token)).await
        .and_then(|res| serde_json::from_value::<Vec<Invoice>>(res).map_err(|e| e.to_string()))
}

pub async fn create_invoice(token: &str, invoice: &Invoice) -> Result<Invoice, String> {
    post("/api/invoices", &json!(invoice), Some(token)).await
        .and_then(|res| serde_json::from_value::<Invoice>(res).map_err(|e| e.to_string()))
}

// ... други API функции
```

### models.rs

Frontend модели (типове) за сериализация.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: bool,
    pub is_super_admin: bool,
    pub email_verified_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: i64,
    pub name: String,
    pub eik: String,
    pub vat_number: String,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub number: String,
    pub name: String,
    pub account_type: String,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counterpart {
    pub id: i64,
    pub name: String,
    pub eik: String,
    pub counterpart_type: String, // "customer" or "supplier"
    pub vat_number: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: i64,
    pub number: String,
    pub date: String,
    pub counterpart_id: i64,
    pub counterpart_name: String,
    pub total_amount: f64,
    pub vat_amount: f64,
    pub paid: bool,
    pub invoice_type: String,
    pub invoice_lines: Vec<InvoiceLine>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLine {
    pub id: i64,
    pub invoice_id: i64,
    pub product_id: i64,
    pub product_name: String,
    pub quantity: f64,
    pub unit: String,
    pub unit_price: f64,
    pub total_price: f64,
    pub vat_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: i64,
    pub invoice_id: i64,
    pub date: String,
    pub amount: f64,
    pub payment_method: String,
    pub bank_account_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: i64,
    pub date: String,
    pub description: String,
    pub journal_lines: Vec<JournalLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalLine {
    pub id: i64,
    pub account_id: i64,
    pub account_name: String,
    pub account_number: String,
    pub debit: f64,
    pub credit: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub unit: String,
    pub price: f64,
    pub vat_rate: f64,
    pub is_service: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedAsset {
    pub id: i64,
    pub name: String,
    pub inventory_number: String,
    pub acquisition_date: String,
    pub initial_value: f64,
    pub depreciation_rate: f64,
    pub accumulated_depreciation: f64,
    pub net_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccount {
    pub id: i64,
    pub name: String,
    pub bank_name: String,
    pub account_number: String,
    pub iban: String,
    pub currency: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankTransaction {
    pub id: i64,
    pub bank_account_id: i64,
    pub date: String,
    pub amount: f64,
    pub counterparty: String,
    pub description: String,
    pub reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Currency {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRate {
    pub id: i64,
    pub currency_id: i64,
    pub currency_code: String,
    pub rate: f64,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VatReturn {
    pub id: i64,
    pub period_start: String,
    pub period_end: String,
    pub vat_sales: f64,
    pub vat_purchases: f64,
    pub vat_payable: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedInvoice {
    pub id: i64,
    pub file_name: String,
    pub status: String,
    pub extracted_data: Option<serde_json::Value>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: User,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub errors: HashMap<String, Vec<String>>,
}
```

### context/

App context за глобално състояние.

```rust
// context/auth_context.rs
#[derive(Clone)]
pub struct AuthContext {
    pub token: Signal<Option<String>>,
    pub user: Signal<Option<User>>,
}

impl AuthContext {
    pub fn new() -> Self {
        let (token, set_token) = create_signal(localStorage::get("token").ok());
        let (user, set_user) = create_signal(
            localStorage::get("user")
                .ok()
                .and_then(|s| serde_json::from_str::<User>(&s).ok())
        );

        Self { token, user }
    }

    pub fn login(&self, response: LoginResponse) {
        self.token.set(Some(response.token.clone()));
        self.user.set(Some(response.user));
        localStorage::set("token", &response.token);
        localStorage::set("user", &serde_json::to_string(&response.user).unwrap());
    }

    pub fn logout(&self) {
        self.token.set(None);
        self.user.set(None);
        localStorage::remove("token");
        localStorage::remove("user");
    }

    pub fn is_authenticated(&self) -> bool {
        self.token.get().is_some()
    }
}
```

### constants/

Константи за приложението.

```rust
pub const API_BASE_URL: &str = "http://localhost:5000";

pub const VAT_RATES: &[f64] = &[0.0, 9.0, 20.0];

pub const INVOICE_TYPES: &[&str] = &["sales", "purchase"];

pub const PAYMENT_METHODS: &[&str] = &["cash", "bank", "card", "transfer"];

pub const ASSET_MOVEMENT_TYPES: &[&str] = &["acquisition", "depreciation", "sale", "disposal"];

pub const STOCK_MOVEMENT_TYPES: &[&str] = &["receipt", "issue", "transfer"];

pub const SAFT_REPORT_TYPES: &[&str] = &["monthly", "on_demand", "annual"];
```

### i18n/

Интернационализация.

```rust
// i18n/bg.json
{
    "app": {
        "title": "Бараба",
        "dashboard": "Табло",
        "invoices": "Фактури",
        "accounts": "Сметкоплан",
        "counterparts": "Контрагенти",
        "products": "Продукти",
        "fixed_assets": "Дълготрайни активи",
        "journal_entries": "Счетоводни записи",
        "payments": "Плащания",
        "saft_export": "SAF-T експорт",
        "vat_returns": "ДДС декларации",
        "currencies": "Валути",
        "users": "Потребители",
        "settings": "Настройки",
        "logout": "Изход"
    },
    "auth": {
        "login": "Вход",
        "register": "Регистрация",
        "forgot_password": "Забравена парола",
        "reset_password": "Смяна на парола",
        "verify_email": "Валидация на email"
    },
    "invoices": {
        "title": "Фактури",
        "new_invoice": "Нова фактура",
        "number": "Номер",
        "date": "Дата",
        "counterpart": "Контрагент",
        "total": "Общо",
        "vat": "ДДС",
        "paid": "Платена",
        "unpaid": "Неплатена"
    }
}
```

Използване в компоненти:
```rust
let t = use_i18n::<Translations>();

view! {
    <h1>{t.invoice.title}</h1>
}
```

## Директория `src/pages/`

### Auth Pages

#### Login (`login.rs`)

```rust
#[component]
pub fn Login() -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);

    let on_submit = move |_| {
        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            match api::login(&email.get(), &password.get()).await {
                Ok(response) => {
                    // Save token and user to localStorage
                    localStorage::set("token", &response.token);
                    localStorage::set("user", &serde_json::to_string(&response.user).unwrap());
                    // Navigate to dashboard
                    window().location().set("/").unwrap();
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="login-page">
            <div class="login-form">
                <h1>"Вход"</h1>
                {move || error.get().map(|e| view! { <div class="alert alert-danger">{e}</div> })}
                <form on:submit=on_submit>
                    <div class="form-group">
                        <label>"Email"</label>
                        <input
                            type="email"
                            prop:value=email
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-group">
                        <label>"Парола"</label>
                        <input
                            type="password"
                            prop:value=password
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                        />
                    </div>
                    <button type="submit" disabled=move || loading.get()>
                        {move || if loading.get() { "Влизане..." } else { "Влез" }}
                    </button>
                </form>
                <p>"Нямате акаунт? <A href="/register">"Регистрирайте се"</A></p>
                <p><A href="/forgot-password">"Забравена парола?"</A></p>
            </div>
        </div>
    }
}
```

#### Register (`register.rs`), Forgot Password (`forgot_password.rs`), Reset Password (`reset_password.rs`), Verify Email (`verify_email.rs`)

Подобна структура като Login.

### Main Pages

#### Dashboard (`dashboard.rs`)

```rust
#[component]
pub fn Dashboard() -> impl IntoView {
    let invoices = create_local_resource(
        || (),
        |_| async move {
            let token = localStorage::get("token").unwrap();
            api::get_invoices(&token).await.unwrap_or_default()
        }
    );

    view! {
        <div class="dashboard">
            <h1>"Табло"</h1>
            <div class="stats">
                <div class="stat-card">
                    <h3>"Фактури"</h3>
                    <p>{move || invoices.get().map(|i| i.len()).unwrap_or(0)}</p>
                </div>
                <div class="stat-card">
                    <h3>"Сметки"</h3>
                    <p>150</p>
                </div>
                <div class="stat-card">
                    <h3>"Контрагенти"</h3>
                    <p>50</p>
                </div>
            </div>
        </div>
    }
}
```

#### Accounts (`accounts.rs`)

Сметкоплан с CRUD операции, търсене, филтриране по тип и пагинация.

Модалът за Нова сметка / Редактиране съдържа:
- Код и наименование
- Тип сметка (активна, пасивна, активно-пасивна, приходна, разходна)
- Чекбокс "Активна сметка"
- Чекбокс "Отчитане на артикули/продукти" (`tracks_articles`)
- Searchable dropdown за SAF-T стандартна сметка (от НАП номенклатура, lazy-load при отваряне на модала)

Таблицата показва: Код, Наименование, Тип, SAF-T сметка, Арт. (да/не), Действия.

При празен сметкоплан - бутон "Зареди стандартен сметкоплан (SAF-T)" за автоматична инициализация от НАП номенклатурата.

#### Invoices (`invoices.rs`), Counterparts (`counterparts.rs`), Journal Entries (`journal_entries.rs`), Payments (`payments.rs`), Products (`products.rs`), Fixed Assets (`fixed_assets.rs`), Bank Accounts (`bank_accounts.rs`), Bank Transactions (`bank_transactions.rs`), Currencies (`currencies.rs`), Exchange Rates (`exchange_rates.rs`), VAT Returns (`vat_returns.rs`), Accounting Periods (`accounting_periods.rs`), Opening Balances (`opening_balances.rs`), Warehouse (`warehouse.rs`), Stock Transactions (`stock_transactions.rs`), Documents (`documents.rs`), Users (`users.rs`), Roles (`roles.rs`), Companies (`companies.rs`)

Подобна структура на CRUD операции с Table компонент.

#### Scanned Invoices (`scanned_invoices.rs`)

```rust
#[component]
pub fn ScannedInvoicesIndex() -> impl IntoView {
    let scanned_invoices = create_local_resource(
        || (),
        |_| async move {
            let token = localStorage::get("token").unwrap();
            api::get_scanned_invoices(&token).await.unwrap_or_default()
        }
    );

    view! {
        <div class="scanned-invoices-page">
            <div class="page-header">
                <h1>"Сканирани фактури"</h1>
                <button class="btn btn-primary">"Качване"</button>
            </div>
            {move || scanned_invoices.read().map(|data| {
                view! {
                    <Table
                        columns=vec![
                            TableColumn {
                                label: "Файл".to_string(),
                                key: "file_name".to_string(),
                                render: |i: serde_json::Value| i["file_name"].as_str().unwrap().to_string()
                            },
                            TableColumn {
                                label: "Статус".to_string(),
                                key: "status".to_string(),
                                render: |i: serde_json::Value| i["status"].as_str().unwrap().to_string()
                            },
                            TableColumn {
                                label: "Дата".to_string(),
                                key: "created_at".to_string(),
                                render: |i: serde_json::Value| i["created_at"].as_str().unwrap().to_string()
                            },
                        ]
                        data=data
                        render_row=|invoice| view! {
                            <tr>
                                <td>{invoice["file_name"].as_str().unwrap()}</td>
                                <td>{invoice["status"].as_str().unwrap()}</td>
                                <td>{invoice["created_at"].as_str().unwrap()}</td>
                                <td>
                                    <button>"Mistral AI"</button>
                                    <button>"Създай фактура"</button>
                                </td>
                            </tr>
                        }
                    />
                }
            })}
        </div>
    }
}
```

#### SAF-T Export (`saft_export.rs`)

```rust
#[component]
pub fn SaftExport() -> impl IntoView {
    let (period_start, set_period_start) = create_signal(String::new());
    let (period_end, set_period_end) = create_signal(String::new());
    let (report_type, set_report_type) = create_signal("monthly".to_string());
    let (loading, set_loading) = create_signal(false);

    let on_export = move |_| {
        set_loading.set(true);

        spawn_local(async move {
            let token = localStorage::get("token").unwrap();
            match api::export_saft(&token, &period_start.get(), &period_end.get(), &report_type.get()).await {
                Ok(xml) => {
                    // Download XML file
                    let blob = Blob::new(vec![xml.as_str()]);
                    let url = Url::create_object_url(&blob);
                    let window = web_sys::window().unwrap();
                    let document = window.document().unwrap();
                    let a = document.create_element("a").unwrap();
                    a.set_attribute("href", &url).unwrap();
                    a.set_attribute("download", "saft_export.xml").unwrap();
                    a.click();
                }
                Err(e) => {
                    // Show error
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="saft-export-page">
            <h1>"SAF-T експорт"</h1>
            <form>
                <div class="form-group">
                    <label>"Начална дата"</label>
                    <input
                        type="date"
                        prop:value=period_start
                        on:input=move |ev| set_period_start.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-group">
                    <label>"Крайна дата"</label>
                    <input
                        type="date"
                        prop:value=period_end
                        on:input=move |ev| set_period_end.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-group">
                    <label>"Тип отчет"</label>
                    <select on:change=move |ev| set_report_type.set(event_target_value(&ev))>
                        <option value="monthly">"Месечен"</option>
                        <option value="on_demand">"При поискване"</option>
                        <option value="annual">"Годишен"</option>
                    </select>
                </div>
                <button type="button" on:click=on_export disabled=move || loading.get()>
                    {move || if loading.get() { "Експортиране..." } else { "Експорт" }}
                </button>
            </form>
        </div>
    }
}
```

#### Controlisy Import (`controlisy_import.rs`), SAFT Movement Mappings (`saft_movement_mappings.rs`), Reports (`reports.rs`), Settings (`settings.rs`), Profile (`profile.rs`), Admin Dashboard (`admin_dashboard.rs`), System Settings (`system_settings.rs`)

Подобна структура.

## Директория `src/components/`

Reusable UI компоненти.

### Table компонент

```rust
#[component]
pub fn Table<T, F>(
    columns: Vec<TableColumn<T>>,
    data: Vec<T>,
    render_row: F,
    #[prop(optional)] loading: bool = false,
) -> impl IntoView
where
    T: Clone + 'static,
    F: Fn(T) -> View + 'static,
{
    view! {
        <table class="table">
            <thead>
                <tr>
                    {columns.iter().map(|col| view! {
                        <th>{col.label.clone()}</th>
                    }).collect_view()}
                </tr>
            </thead>
            <tbody>
                {if loading {
                    view! { <tr><td colspan={columns.len()}>"Зареждане..."</td></tr> }
                } else if data.is_empty() {
                    view! { <tr><td colspan={columns.len()}>"Няма данни"</td></tr> }
                } else {
                    data.into_iter().map(render_row).collect_view()
                }}
            </tbody>
        </table>
    }
}

#[derive(Clone)]
pub struct TableColumn<T> {
    pub label: String,
    pub key: String,
    pub render: fn(T) -> String,
}
```

### Form компонент

```rust
#[component]
pub fn FormInput<T>(
    label: String,
    #[prop(default)] id: Option<String>,
    #[prop(default)] name: Option<String>,
    #[prop(default)] value: Signal<T>,
    #[prop(default)] on_change: Option<Callback<T>>,
    #[prop(default)] error: Option<Signal<Option<String>>>,
) -> impl IntoView
where
    T: ToString + FromStr + Clone + 'static,
{
    view! {
        <div class="form-group">
            <label for=id.clone()>{label}</label>
            <input
                type="text"
                id=id
                name=name
                class:is-invalid=move || error.as_ref().map(|e| e.get().is_some()).unwrap_or(false)
                prop:value=move || value.get().to_string()
                on:input=move |ev| {
                    let new_value = T::from_str(&ev.target().value()).unwrap_or_else(|_| value.get());
                    value.set(new_value);
                    if let Some(ref cb) = on_change {
                        cb.call(new_value);
                    }
                }
            />
            {move || error.as_ref().and_then(|e| e.get()).map(|err| view! { <div class="invalid-feedback">{err}</div> })}
        </div>
    }
}
```

### Modal компонент

```rust
#[component]
pub fn Modal(
    show: Signal<bool>,
    title: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class:modal=move || show.get()>
            <div class="modal-backdrop" on:click=move |_| show.set(false)></div>
            <div class="modal-content">
                <div class="modal-header">
                    <h2>{title}</h2>
                    <button class="close" on:click=move |_| show.set(false)>"×"</button>
                </div>
                <div class="modal-body">
                    {children()}
                </div>
            </div>
        </div>
    }
}
```

### Pagination компонент

```rust
#[component]
pub fn Pagination(
    page: Signal<usize>,
    total_pages: usize,
    on_page_change: Callback<usize>,
) -> impl IntoView {
    view! {
        <div class="pagination">
            <button
                disabled=move || page.get() == 0
                on:click=move |_| on_page_change.call(page.get() - 1)
            >
                "Предишна"
            </button>
            <span>
                {move || page.get() + 1}
                " / "
                {total_pages}
            </span>
            <button
                disabled=move || page.get() >= total_pages - 1
                on:click=move |_| on_page_change.call(page.get() + 1)
            >
                "Следваща"
            </button>
        </div>
    }
}
```

## Директория `src/stores/`

State management.

```rust
// stores/auth_store.rs
#[derive(Clone)]
pub struct AuthStore {
    pub token: Signal<Option<String>>,
    pub user: Signal<Option<User>>,
}

impl AuthStore {
    pub fn new() -> Self {
        let (token, set_token) = create_signal(localStorage::get("token").ok());
        let (user, set_user) = create_signal(
            localStorage::get("user")
                .ok()
                .and_then(|s| serde_json::from_str::<User>(&s).ok())
        );

        // Save to localStorage on change
        create_effect(move |_| {
            if let Some(ref t) = token.get() {
                localStorage::set("token", t);
            }
        });

        create_effect(move |_| {
            if let Some(ref u) = user.get() {
                localStorage::set("user", &serde_json::to_string(u).unwrap());
            }
        });

        Self { token, user }
    }

    pub fn login(&self, response: LoginResponse) {
        self.token.set(Some(response.token.clone()));
        self.user.set(Some(response.user));
    }

    pub fn logout(&self) {
        self.token.set(None);
        self.user.set(None);
        localStorage::remove("token");
        localStorage::remove("user");
    }

    pub fn is_authenticated(&self) -> bool {
        self.token.get().is_some()
    }
}
```

## CSS Styling

Стиловете се намират в `assets/` и `style/`.

### assets/

Tailwind CSS или други CSS файлове.

### style/

Глобални стилове.

```css
/* style/main.css */
:root {
    --primary-color: #007bff;
    --secondary-color: #6c757d;
    --success-color: #28a745;
    --danger-color: #dc3545;
    --warning-color: #ffc107;
    --info-color: #17a2b8;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    margin: 0;
    padding: 0;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
}

.btn {
    padding: 10px 20px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
}

.btn-primary {
    background-color: var(--primary-color);
    color: white;
}

/* ... */
```

## Deployment

### Build for production

```bash
cd leptos
trunk build --release
```

Резултатът е в `leptos/dist/`, който може да бъде deployment-нат като статичен сайт.

### Сървинг на WASM файловете

Trunk генерира оптимизирани WASM файлове. Уверете се, че сървърът:
- Сърва `.wasm` файлове с `Content-Type: application/wasm`
- Сърва `.gz` файлове с `Content-Encoding: gzip`
- Има appropriate CORS настройки

### Environment variables

Може да добавите environment-specific конфигурация в `Cargo.toml` или чрез build.rs.

## Полезни ресурси

- [Leptos documentation](https://leptos.dev/)
- [Rust WASM book](https://rustwasm.github.io/docs/book/)
- [Trunk documentation](https://trunkrs.dev/)
- [gloo-net](https://docs.rs/gloo-net/)

---

**Последна актуализация:** 14 Февруари 2026
