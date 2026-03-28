use leptos::*;
use web_sys::window;

pub type TranslateFn = &'static str;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    BG,
    EN,
}

#[derive(Clone, Copy)]
pub struct I18n {
    pub lang: Language,
}

impl I18n {
    pub fn new() -> Self {
        let lang = Self::load_from_storage();
        Self { lang }
    }

    pub fn set_language(&mut self, lang: Language) {
        self.lang = lang;
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("language", self.lang_code());
            }
        }
    }

    pub fn toggle_language(&mut self) {
        self.lang = match self.lang {
            Language::BG => Language::EN,
            Language::EN => Language::BG,
        };
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("language", self.lang_code());
            }
        }
    }

    pub fn lang_code(&self) -> &'static str {
        match self.lang {
            Language::BG => "bg",
            Language::EN => "en",
        }
    }

    fn load_from_storage() -> Language {
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(lang)) = storage.get_item("language") {
                    return match lang.as_str() {
                        "bg" => Language::BG,
                        "en" => Language::EN,
                        _ => Language::BG,
                    };
                }
            }
        }
        Language::BG
    }

    pub fn t(&self, key: &str) -> String {
        match self.lang {
            Language::BG => self.t_bg(key),
            Language::EN => self.t_en(key),
        }
    }

    fn t_bg(&self, key: &str) -> String {
        match key {
            "common.save" => "Запази".to_string(),
            "common.cancel" => "Отказ".to_string(),
            "common.delete" => "Изтрий".to_string(),
            "common.edit" => "Редактирай".to_string(),
            "common.add" => "Добави".to_string(),
            "common.search" => "Търсене".to_string(),
            "common.clear" => "Изчисти".to_string(),
            "common.loading" => "Зареждане...".to_string(),
            "common.error" => "Грешка".to_string(),
            "common.success" => "Успех".to_string(),
            "common.confirm" => "Потвърди".to_string(),
            "common.close" => "Затвори".to_string(),
            "common.yes" => "Да".to_string(),
            "common.no" => "Не".to_string(),
            "common.actions" => "Действия".to_string(),
            "common.status" => "Статус".to_string(),
            "common.name" => "Име".to_string(),
            "common.type" => "Тип".to_string(),
            "common.date" => "Дата".to_string(),
            "common.amount" => "Сума".to_string(),
            "common.noData" => "Няма данни".to_string(),
            "common.refresh" => "Обнови".to_string(),

            "auth.login" => "Вход".to_string(),
            "auth.logout" => "Изход".to_string(),
            "auth.register" => "Регистрация".to_string(),
            "auth.email" => "Имейл".to_string(),
            "auth.password" => "Парола".to_string(),
            "auth.confirmPassword" => "Потвърди парола".to_string(),
            "auth.invalidCredentials" => "Невалидни данни за вход".to_string(),
            "auth.networkError" => "Грешка в мрежата".to_string(),
            "auth.noAccount" => "Нямате акаунт?".to_string(),
            "auth.haveAccount" => "Имате акаунт?".to_string(),
            "auth.registerSuccess" => "Регистрацията е успешна!".to_string(),
            "auth.registerError" => "Грешка при регистрация".to_string(),
            "auth.passwordMismatch" => "Паролите не съвпадат".to_string(),
            "auth.welcome" => "Добре дошли".to_string(),
            "auth.subtitle" => "Счетоводна система".to_string(),

            "nav.dashboard" => "Табло".to_string(),
            "nav.invoices" => "Фактури".to_string(),
            "nav.accounts" => "Сметки".to_string(),
            "nav.counterparts" => "Контрагенти".to_string(),
            "nav.products" => "Продукти".to_string(),
            "nav.settings" => "Настройки".to_string(),
            "nav.companies" => "Фирми".to_string(),
            "nav.journalEntries" => "Счетоводни записи".to_string(),
            "nav.controlisyImport" => "Импорт Controlisy".to_string(),
            "nav.vatReturns" => "ДДС Дневници".to_string(),

            "dashboard.title" => "Табло".to_string(),
            "dashboard.totalInvoices" => "Общо фактури".to_string(),
            "dashboard.revenue" => "Приходи".to_string(),
            "dashboard.expenses" => "Разходи".to_string(),
            "dashboard.counterparts" => "Контрагенти".to_string(),
            "dashboard.recentInvoices" => "Последни фактури".to_string(),
            "dashboard.selectCompany" => "Изберете фирма".to_string(),

            "invoices.title" => "Фактури".to_string(),
            "invoices.addNew" => "Нова фактура".to_string(),
            "invoices.number" => "Номер".to_string(),
            "invoices.issueDate" => "Дата на издаване".to_string(),
            "invoices.dueDate" => "Падеж".to_string(),
            "invoices.counterpart" => "Контрагент".to_string(),
            "invoices.totalAmount" => "Обща сума".to_string(),
            "invoices.vatAmount" => "ДДС".to_string(),
            "invoices.status" => "Статус".to_string(),
            "invoices.draft" => "Чернова".to_string(),
            "invoices.posted" => "Осчетоводена".to_string(),
            "invoices.invoice" => "Фактура".to_string(),
            "invoices.creditNote" => "Кредитно известие".to_string(),
            "invoices.debitNote" => "Дебитно известие".to_string(),
            "invoices.noInvoices" => "Няма фактури".to_string(),
            "invoices.post" => "Осчетоводи".to_string(),

            "counterparts.title" => "Контрагенти".to_string(),
            "counterparts.addNew" => "Нов контрагент".to_string(),
            "counterparts.name" => "Име".to_string(),
            "counterparts.vatNumber" => "ДДС номер".to_string(),
            "counterparts.address" => "Адрес".to_string(),
            "counterparts.contactPerson" => "МОЛ".to_string(),
            "counterparts.email" => "Имейл".to_string(),
            "counterparts.phone" => "Телефон".to_string(),
            "counterparts.type" => "Тип".to_string(),
            "counterparts.customer" => "Клиент".to_string(),
            "counterparts.supplier" => "Доставчик".to_string(),
            "counterparts.noCounterparts" => "Няма контрагенти".to_string(),

            "accounts.title" => "Сметкоплан".to_string(),
            "accounts.addNew" => "Нова сметка".to_string(),
            "accounts.code" => "Код".to_string(),
            "accounts.name" => "Наименование".to_string(),
            "accounts.type" => "Тип".to_string(),
            "accounts.asset" => "Актив".to_string(),
            "accounts.liability" => "Пасив".to_string(),
            "accounts.equity" => "Капитал".to_string(),
            "accounts.revenue" => "Приход".to_string(),
            "accounts.expense" => "Разход".to_string(),
            "accounts.noAccounts" => "Няма сметки".to_string(),

            "journal.title" => "Счетоводни записи".to_string(),
            "journal.newEntry" => "Нов запис".to_string(),
            "journal.editEntry" => "Редактиране на запис".to_string(),
            "journal.noEntries" => "Няма записи".to_string(),
            "journal.purchase" => "Покупка".to_string(),
            "journal.sale" => "Продажба".to_string(),
            "journal.nonVat" => "Без ДДС".to_string(),
            "journal.description" => "Описание".to_string(),
            "journal.debit" => "Дебит".to_string(),
            "journal.credit" => "Кредит".to_string(),

            "vat.title" => "ДДС Дневници и Декларации".to_string(),
            "vat.purchases" => "Покупки".to_string(),
            "vat.sales" => "Продажби".to_string(),
            "vat.noReturns" => "Няма декларации".to_string(),

            "nav.saftExport" => "SAF-T Експорт".to_string(),
            "saft.title" => "SAF-T BG Експорт".to_string(),
            "saft.infoTitle" => "SAF-T BG v1.0.1".to_string(),
            "saft.infoText" => "Стандартен одитен файл за данъчни цели (Standard Audit File for Tax). Генерира XML файл съгласно българските изисквания за подаване към НАП.".to_string(),
            "saft.selectPeriod" => "Изберете период".to_string(),
            "saft.company" => "Фирма".to_string(),
            "saft.year" => "Година".to_string(),
            "saft.month" => "Месец".to_string(),
            "saft.reportType" => "Тип отчет".to_string(),
            "saft.typeMonthly" => "Месечен".to_string(),
            "saft.typeOnDemand" => "По искане".to_string(),
            "saft.typeAnnual" => "Годишен".to_string(),
            "saft.validate" => "Валидирай".to_string(),
            "saft.export" => "Експорт XML".to_string(),
            "saft.selectCompanyFirst" => "Моля, изберете фирма".to_string(),
            "saft.validationFailed" => "Грешка при валидация".to_string(),
            "saft.exportFailed" => "Грешка при експорт".to_string(),
            "saft.validationResults" => "Резултати от валидация".to_string(),
            "saft.journalEntries" => "Счетоводни записи".to_string(),
            "saft.invoices" => "Фактури".to_string(),
            "saft.counterparts" => "Контрагенти".to_string(),
            "saft.accounts" => "Сметки".to_string(),
            "saft.errors" => "Грешки".to_string(),
            "saft.warnings" => "Предупреждения".to_string(),
            "saft.allValid" => "Всички данни са валидни и готови за експорт!".to_string(),

            _ => key.to_string(),
        }
    }

    fn t_en(&self, key: &str) -> String {
        match key {
            "common.save" => "Save".to_string(),
            "common.cancel" => "Cancel".to_string(),
            "common.delete" => "Delete".to_string(),
            "common.edit" => "Edit".to_string(),
            "common.add" => "Add".to_string(),
            "common.search" => "Search".to_string(),
            "common.clear" => "Clear".to_string(),
            "common.loading" => "Loading...".to_string(),
            "common.error" => "Error".to_string(),
            "common.success" => "Success".to_string(),
            "common.confirm" => "Confirm".to_string(),
            "common.close" => "Close".to_string(),
            "common.yes" => "Yes".to_string(),
            "common.no" => "No".to_string(),
            "common.actions" => "Actions".to_string(),
            "common.status" => "Status".to_string(),
            "common.name" => "Name".to_string(),
            "common.type" => "Type".to_string(),
            "common.date" => "Date".to_string(),
            "common.amount" => "Amount".to_string(),
            "common.noData" => "No data".to_string(),
            "common.refresh" => "Refresh".to_string(),

            "auth.login" => "Sign In".to_string(),
            "auth.logout" => "Logout".to_string(),
            "auth.register" => "Sign Up".to_string(),
            "auth.email" => "Email".to_string(),
            "auth.password" => "Password".to_string(),
            "auth.confirmPassword" => "Confirm Password".to_string(),
            "auth.invalidCredentials" => "Invalid credentials".to_string(),
            "auth.networkError" => "Network error".to_string(),
            "auth.noAccount" => "Don't have an account?".to_string(),
            "auth.haveAccount" => "Already have an account?".to_string(),
            "auth.registerSuccess" => "Registration successful!".to_string(),
            "auth.registerError" => "Registration failed".to_string(),
            "auth.passwordMismatch" => "Passwords don't match".to_string(),
            "auth.welcome" => "Welcome".to_string(),
            "auth.subtitle" => "Accounting System".to_string(),

            "nav.dashboard" => "Dashboard".to_string(),
            "nav.invoices" => "Invoices".to_string(),
            "nav.accounts" => "Accounts".to_string(),
            "nav.counterparts" => "Counterparts".to_string(),
            "nav.products" => "Products".to_string(),
            "nav.settings" => "Settings".to_string(),
            "nav.companies" => "Companies".to_string(),
            "nav.journalEntries" => "Journal Entries".to_string(),
            "nav.controlisyImport" => "Controlisy Import".to_string(),
            "nav.vatReturns" => "VAT Returns".to_string(),

            "dashboard.title" => "Dashboard".to_string(),
            "dashboard.totalInvoices" => "Total Invoices".to_string(),
            "dashboard.revenue" => "Revenue".to_string(),
            "dashboard.expenses" => "Expenses".to_string(),
            "dashboard.counterparts" => "Counterparts".to_string(),
            "dashboard.recentInvoices" => "Recent Invoices".to_string(),
            "dashboard.selectCompany" => "Select company".to_string(),

            "invoices.title" => "Invoices".to_string(),
            "invoices.addNew" => "New Invoice".to_string(),
            "invoices.number" => "Number".to_string(),
            "invoices.issueDate" => "Issue Date".to_string(),
            "invoices.dueDate" => "Due Date".to_string(),
            "invoices.counterpart" => "Counterpart".to_string(),
            "invoices.totalAmount" => "Total Amount".to_string(),
            "invoices.vatAmount" => "VAT".to_string(),
            "invoices.status" => "Status".to_string(),
            "invoices.draft" => "Draft".to_string(),
            "invoices.posted" => "Posted".to_string(),
            "invoices.invoice" => "Invoice".to_string(),
            "invoices.creditNote" => "Credit Note".to_string(),
            "invoices.debitNote" => "Debit Note".to_string(),
            "invoices.noInvoices" => "No invoices".to_string(),
            "invoices.post" => "Post".to_string(),

            "counterparts.title" => "Counterparts".to_string(),
            "counterparts.addNew" => "New Counterpart".to_string(),
            "counterparts.name" => "Name".to_string(),
            "counterparts.vatNumber" => "VAT Number".to_string(),
            "counterparts.address" => "Address".to_string(),
            "counterparts.contactPerson" => "Contact Person".to_string(),
            "counterparts.email" => "Email".to_string(),
            "counterparts.phone" => "Phone".to_string(),
            "counterparts.type" => "Type".to_string(),
            "counterparts.customer" => "Customer".to_string(),
            "counterparts.supplier" => "Supplier".to_string(),
            "counterparts.noCounterparts" => "No counterparts".to_string(),

            "accounts.title" => "Chart of Accounts".to_string(),
            "accounts.addNew" => "New Account".to_string(),
            "accounts.code" => "Code".to_string(),
            "accounts.name" => "Name".to_string(),
            "accounts.type" => "Type".to_string(),
            "accounts.asset" => "Asset".to_string(),
            "accounts.liability" => "Liability".to_string(),
            "accounts.equity" => "Equity".to_string(),
            "accounts.revenue" => "Revenue".to_string(),
            "accounts.expense" => "Expense".to_string(),
            "accounts.noAccounts" => "No accounts".to_string(),

            "journal.title" => "Journal Entries".to_string(),
            "journal.newEntry" => "New Entry".to_string(),
            "journal.editEntry" => "Edit Entry".to_string(),
            "journal.noEntries" => "No entries".to_string(),
            "journal.purchase" => "Purchase".to_string(),
            "journal.sale" => "Sale".to_string(),
            "journal.nonVat" => "Non-VAT".to_string(),
            "journal.description" => "Description".to_string(),
            "journal.debit" => "Debit".to_string(),
            "journal.credit" => "Credit".to_string(),

            "vat.title" => "VAT Returns".to_string(),
            "vat.purchases" => "Purchases".to_string(),
            "vat.sales" => "Sales".to_string(),
            "vat.noReturns" => "No returns".to_string(),

            "nav.saftExport" => "SAF-T Export".to_string(),
            "saft.title" => "SAF-T BG Export".to_string(),
            "saft.infoTitle" => "SAF-T BG v1.0.1".to_string(),
            "saft.infoText" => "Standard Audit File for Tax. Generates an XML file according to Bulgarian requirements for submission to NRA (National Revenue Agency).".to_string(),
            "saft.selectPeriod" => "Select Period".to_string(),
            "saft.company" => "Company".to_string(),
            "saft.year" => "Year".to_string(),
            "saft.month" => "Month".to_string(),
            "saft.reportType" => "Report Type".to_string(),
            "saft.typeMonthly" => "Monthly".to_string(),
            "saft.typeOnDemand" => "On Demand".to_string(),
            "saft.typeAnnual" => "Annual".to_string(),
            "saft.validate" => "Validate".to_string(),
            "saft.export" => "Export XML".to_string(),
            "saft.selectCompanyFirst" => "Please select a company".to_string(),
            "saft.validationFailed" => "Validation failed".to_string(),
            "saft.exportFailed" => "Export failed".to_string(),
            "saft.validationResults" => "Validation Results".to_string(),
            "saft.journalEntries" => "Journal Entries".to_string(),
            "saft.invoices" => "Invoices".to_string(),
            "saft.counterparts" => "Counterparts".to_string(),
            "saft.accounts" => "Accounts".to_string(),
            "saft.errors" => "Errors".to_string(),
            "saft.warnings" => "Warnings".to_string(),
            "saft.allValid" => "All data is valid and ready for export!".to_string(),

            _ => key.to_string(),
        }
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}

pub fn use_i18n() -> I18n {
    I18n::new()
}
