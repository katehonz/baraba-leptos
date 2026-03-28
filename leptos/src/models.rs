use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: i64,
    pub name: String,
    pub eik: String,
    pub vat_number: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub post_code: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub manager_name: Option<String>,
    pub manager_eik: Option<String>,
    pub accountant_name: Option<String>,
    pub accountant_egn: Option<String>,
    pub tax_authority: Option<String>,
    pub inventory_valuation_method: Option<String>,
    pub is_vat_registered: bool,
    pub nap_office: Option<String>,
    pub vat_period: String,
    pub currency: String,
    pub fiscal_year_start_month: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub is_active: bool,
    pub company_id: i64,
    pub saft_account_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaftAccount {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub section_code: String,
    pub section_name: String,
    pub group_code: String,
    pub group_name: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counterpart {
    pub id: i64,
    pub name: String,
    pub counterpart_type: String,
    pub vat_number: Option<String>,
    pub eik: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub post_code: Option<String>,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLine {
    pub id: Option<i64>,
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub vat_rate: f64,
    pub vat_amount: f64,
    pub total: f64,
    pub account_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: i64,
    pub invoice_number: String,
    pub document_type: String,
    pub original_invoice_id: Option<i64>,
    pub issue_date: String,
    pub due_date: Option<String>,
    pub total_net_amount: f64,
    pub total_vat_amount: f64,
    pub total_amount: f64,
    pub currency_code: String,
    pub exchange_rate: f64,
    pub payment_method: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub tax_event_date: Option<String>,
    pub vat_exemption_reason: Option<String>,
    pub has_inventory: bool,
    pub lines: Vec<InvoiceLine>,
    pub company_id: i64,
    pub counterpart_id: i64,
    pub journal_entry_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntryLine {
    pub id: Option<i64>,
    pub account_id: i64,
    pub debit: f64,
    pub credit: f64,
    pub description: Option<String>,
    pub counterpart_id: Option<i64>,
    pub vat_amount: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: i64,
    pub date: String,
    pub description: String,
    pub status: String,
    pub document_number: Option<String>,
    pub document_date: Option<String>,
    pub vat_purchase_operation: Option<String>,
    pub vat_sales_operation: Option<String>,
    pub total_amount: Option<f64>,
    pub total_vat_amount: Option<f64>,
    pub payment_method_code: Option<String>,
    pub vat_period: Option<String>,
    pub lines: Vec<JournalEntryLine>,
    pub company_id: i64,
    pub user_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VatReturn {
    pub id: i64,
    pub period_year: i32,
    pub period_month: i32,
    pub status: String,
    pub vat_due: Option<f64>,
    pub result: Option<String>,
    pub company_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VatJournalEntry {
    pub id: i64,
    pub row_number: i32,
    pub branch_code: Option<String>,
    pub document_type: Option<String>,
    pub document_type_name: Option<String>,
    pub document_number: Option<String>,
    pub document_date: Option<String>,
    pub formatted_date: Option<String>,
    pub counterpart_vat: Option<String>,
    pub counterpart_name: Option<String>,
    pub goods_description: Option<String>,
    pub delivery_code: Option<String>,
    pub entry_type: Option<String>,
    pub base_no_credit: Option<f64>,
    pub base_full_credit: Option<f64>,
    pub vat_full_credit: Option<f64>,
    pub base_partial_credit: Option<f64>,
    pub vat_partial_credit: Option<f64>,
    pub annual_adjustment_base: Option<f64>,
    pub annual_adjustment_vat: Option<f64>,
    pub base20: Option<f64>,
    pub vat20: Option<f64>,
    pub vop_base: Option<f64>,
    pub vop_vat: Option<f64>,
    pub base9: Option<f64>,
    pub vat9: Option<f64>,
    pub total_base: Option<f64>,
    pub total_vat: Option<f64>,
    pub sales_base20: Option<f64>,
    pub sales_vat20: Option<f64>,
    pub sales_base_vop: Option<f64>,
    pub sales_vat_vop: Option<f64>,
    pub sales_base9: Option<f64>,
    pub sales_vat9: Option<f64>,
    pub sales_base0_chapter3: Option<f64>,
    pub sales_base_vod: Option<f64>,
    pub sales_base0_articles: Option<f64>,
    pub sales_base_services21: Option<f64>,
    pub sales_base69_2: Option<f64>,
    pub sales_base69_2_eu: Option<f64>,
    pub sales_base_exempt: Option<f64>,
    pub sales_vat_personal: Option<f64>,
    pub sales_base_vop9: Option<f64>,
    pub source_journal_entry_id: Option<i64>,
    pub notes: Option<String>,
    pub vat_return_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWrapper<T> {
    pub user: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}
