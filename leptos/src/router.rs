use crate::pages::*;
use leptos::*;
use leptos_router::*;

#[component]
pub fn AppRouter() -> impl IntoView {
    view! {
        <Router>
            <main>
                <Routes>
                    <Route path="/" view=Login />
                    <Route path="/login" view=Login />
                    <Route path="/register" view=Register />
                    <Route path="/forgot-password" view=ForgotPassword />
                    <Route path="/reset-password" view=ResetPassword />
                    <Route path="/verify-email" view=VerifyEmail />
                    <Route path="/dashboard" view=Dashboard />
                    <Route path="/journal-entries" view=JournalEntries />
                    <Route path="/invoices" view=Invoices />
                    <Route path="/accounts" view=Accounts />
                    <Route path="/counterparts" view=Counterparts />
                    <Route path="/products" view=Products />
                    <Route path="/settings" view=Settings />
                    <Route path="/system-settings" view=SystemSettings />
                    <Route path="/controlisy-import" view=ControlisyImport />
                    <Route path="/vat-returns" view=VatReturns />
                    <Route path="/fixed-assets" view=FixedAssets />
                    <Route path="/users" view=Users />
                    <Route path="/companies" view=Companies />
                    <Route path="/currencies" view=Currencies />
                    <Route path="/saft-export" view=SaftExport />
                    <Route path="/saft-movement-mappings" view=SaftMovementMappings />
                    <Route path="/opening-balances" view=OpeningBalances />
                    <Route path="/accounting-periods" view=AccountingPeriods />
                    <Route path="/reports" view=ReportsPage />
                    <Route path="/scanned-invoices" view=ScannedInvoices />
                    <Route path="/bank-accounts" view=BankAccounts />
                    <Route path="/bank-transactions" view=BankTransactions />
                    <Route path="/profile" view=Profile />
                    <Route path="/admin" view=AdminDashboard />
                    <Route path="/roles" view=Roles />
                    <Route path="/audit-logs" view=AuditLogs />
                    <Route path="/dividends" view=Dividends />
                    <Route path="/*any" view=NotFound />
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
