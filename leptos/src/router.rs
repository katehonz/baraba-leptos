use crate::pages::*;
use leptos::prelude::*;
use leptos_router::{components::*, StaticSegment, ParamSegment};

#[component]
pub fn AppRouter() -> impl IntoView {
    view! {
        <Router>
            <main>
                <Routes fallback=NotFound>
                    <Route path=StaticSegment("") view=Login />
                    <Route path=StaticSegment("login") view=Login />
                    <Route path=StaticSegment("register") view=Register />
                    <Route path=StaticSegment("forgot-password") view=ForgotPassword />
                    <Route path=StaticSegment("reset-password") view=ResetPassword />
                    <Route path=StaticSegment("verify-email") view=VerifyEmail />
                    <Route path=StaticSegment("dashboard") view=Dashboard />
                    <Route path=StaticSegment("journal-entries") view=JournalEntries />
                    <Route path=StaticSegment("invoices") view=Invoices />
                    <Route path=(StaticSegment("invoices"), ParamSegment("id"), StaticSegment("print")) view=InvoicePrint />
                    <Route path=(StaticSegment("warehouse"), StaticSegment("scan")) view=WarehouseScan />
                    <Route path=StaticSegment("warehouse") view=Warehouse />
                    <Route path=StaticSegment("accounts") view=Accounts />
                    <Route path=StaticSegment("counterparts") view=Counterparts />
                    <Route path=StaticSegment("products") view=Products />
                    <Route path=StaticSegment("settings") view=Settings />
                    <Route path=StaticSegment("system-settings") view=SystemSettings />
                    <Route path=StaticSegment("controlisy-import") view=ControlisyImport />
                    <Route path=StaticSegment("vat-returns") view=VatReturns />
                    <Route path=StaticSegment("fixed-assets") view=FixedAssets />
                    <Route path=StaticSegment("users") view=Users />
                    <Route path=StaticSegment("companies") view=Companies />
                    <Route path=StaticSegment("currencies") view=Currencies />
                    <Route path=StaticSegment("saft-export") view=SaftExport />
                    <Route path=StaticSegment("saft-movement-mappings") view=SaftMovementMappings />
                    <Route path=StaticSegment("opening-balances") view=OpeningBalances />
                    <Route path=StaticSegment("accounting-periods") view=AccountingPeriods />
                    <Route path=StaticSegment("reports") view=ReportsPage />
                    <Route path=StaticSegment("scanned-invoices") view=ScannedInvoices />
                    <Route path=StaticSegment("bank-accounts") view=BankAccounts />
                    <Route path=StaticSegment("bank-transactions") view=BankTransactions />
                    <Route path=StaticSegment("bank-matching-rules") view=BankMatchingRules />
                    <Route path=StaticSegment("profile") view=Profile />
                    <Route path=StaticSegment("admin") view=AdminDashboard />
                    <Route path=StaticSegment("roles") view=Roles />
                    <Route path=StaticSegment("audit-logs") view=AuditLogs />
                    <Route path=StaticSegment("dividends") view=Dividends />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="container">
            <h1>"404 - Page Not Found"</h1>
            <a href="/dashboard">"Back to Dashboard"</a>
        </div>
    }
}
