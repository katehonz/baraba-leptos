/// VAT Codes and Nomenclatures
/// Document types and delivery codes per NAP requirements (ППЗДДС)

#[derive(Debug, Clone)]
pub struct CodeEntry {
    pub code: &'static str,
    pub name: &'static str,
}

impl CodeEntry {
    pub const fn new(code: &'static str, name: &'static str) -> Self {
        Self { code, name }
    }
}

/// Purchase journal document types. PPDDS_2025, field 03-05.
pub const PURCHASE_DOC_TYPES: &[CodeEntry] = &[
    CodeEntry::new("01", "Фактура"),
    CodeEntry::new("02", "Дебитно известие"),
    CodeEntry::new("03", "Кредитно известие"),
    CodeEntry::new("05", "Регистър на стоки до поискване, получени в страната"),
    CodeEntry::new("07", "Митническа декларация"),
    CodeEntry::new("09", "Протокол или друг документ"),
    CodeEntry::new("11", "Фактура — касова отчетност"),
    CodeEntry::new("12", "Дебитно известие — касова отчетност"),
    CodeEntry::new("13", "Кредитно известие — касова отчетност"),
    CodeEntry::new("23", "Кредитно известие по чл. 126б, ал. 1"),
    CodeEntry::new("91", "Протокол за изискуем данък по чл. 151в, ал. 3"),
    CodeEntry::new("92", "Протокол по чл. 151г, ал. 8 или отчет по чл. 104ж, ал. 11"),
    CodeEntry::new("93", "Протокол по чл. 151в, ал. 7 — получател без специален режим"),
    CodeEntry::new("94", "Протокол по чл. 151в, ал. 7 — получател със специален режим"),
];

/// Sales journal document types. PPDDS_2025, field 02-04.
pub const SALES_DOC_TYPES: &[CodeEntry] = &[
    CodeEntry::new("01", "Фактура"),
    CodeEntry::new("02", "Дебитно известие"),
    CodeEntry::new("03", "Кредитно известие"),
    CodeEntry::new("04", "Регистър на стоки до поискване, изпратени към друга държава членка"),
    CodeEntry::new("07", "Митническа декларация"),
    CodeEntry::new("09", "Протокол или друг документ"),
    CodeEntry::new("11", "Фактура — касова отчетност"),
    CodeEntry::new("12", "Дебитно известие — касова отчетност"),
    CodeEntry::new("13", "Кредитно известие — касова отчетност"),
    CodeEntry::new("23", "Кредитно известие по чл. 126б, ал. 1"),
    CodeEntry::new("29", "Протокол по чл. 126б, ал. 2 и 7"),
    CodeEntry::new("50", "Протокол за ДДС върху горива с компенсации"),
    CodeEntry::new("81", "Отчет за извършените продажби"),
    CodeEntry::new("82", "Отчет за продажби при специален ред на облагане"),
    CodeEntry::new("83", "Отчет за продажби при компенсации за горива"),
    CodeEntry::new("84", "Отчет за продажби на хляб"),
    CodeEntry::new("85", "Отчет за продажби на брашно"),
    CodeEntry::new("91", "Протокол за изискуем данък по чл. 151в, ал. 3"),
    CodeEntry::new("93", "Протокол по чл. 151в, ал. 7 — получател без специален режим"),
    CodeEntry::new("94", "Протокол по чл. 151в, ал. 7 — получател със специален режим"),
    CodeEntry::new("95", "Протокол за безвъзмездни хранителни стоки по чл. 6, ал. 4, т. 4"),
];

/// Documents that are posted in the ledger and never enter a VAT journal.
pub const NON_VAT_DOC_TYPES: &[CodeEntry] = &[
    CodeEntry::new("PAYROLL", "Заплати"),
    CodeEntry::new("BANK", "Банков ордер"),
    CodeEntry::new("CASH_IN", "Приходен касов ордер"),
    CodeEntry::new("CASH_OUT", "Разходен касов ордер"),
    CodeEntry::new("SCRAP", "Брак"),
    CodeEntry::new("OTHER", "Друго"),
];

/// Delivery codes for column 8а
pub const DELIVERY_CODES: &[CodeEntry] = &[
    CodeEntry::new("", "Няма"),
    CodeEntry::new("01", "Доставка по част I Приложение 2"),
    CodeEntry::new("02", "Доставка по част II Приложение 2"),
    CodeEntry::new("03", "Внос по Приложение 3"),
    CodeEntry::new("07", "Доставка по чл.163а"),
    CodeEntry::new("08", "Доставка по чл.163б"),
    CodeEntry::new("41", "Изпращане стоки режим складиране"),
    CodeEntry::new("43", "Замяна на лицето чл.15а ал.4"),
    CodeEntry::new("46", "Корекция грешен ИН"),
    CodeEntry::new("48", "Връщане на стоки чл.15а ал.5"),
    CodeEntry::new("51", "Пристигане стоки режим складиране"),
    CodeEntry::new("53", "Замяна без прекратяване"),
    CodeEntry::new("54", "Брак/липса/унищожаване"),
    CodeEntry::new("58", "Прекратяване на договора"),
];

/// Document codes excluded from declaration totals
pub const EXCLUDED_FROM_DECLARATION: &[&str] = &["04", "05", "11", "12", "13", "94"];

/// Purchase, sale, or non-VAT list. `entry_type` is PURCHASE, SALE, or NON_VAT.
pub fn doc_types_for(entry_type: &str) -> &'static [CodeEntry] {
    match entry_type {
        "PURCHASE" | "purchase" => PURCHASE_DOC_TYPES,
        "NON_VAT" | "non_vat" => NON_VAT_DOC_TYPES,
        _ => SALES_DOC_TYPES,
    }
}

pub fn is_non_vat_doc(code: &str) -> bool {
    NON_VAT_DOC_TYPES.iter().any(|entry| entry.code == code)
}

/// Internal invoice kind used for numbering and posting.
pub fn invoice_kind_for_nap(code: &str) -> &'static str {
    match code {
        "PROFORMA" => "PROFORMA",
        "02" | "12" => "DEBIT_NOTE",
        "03" | "13" | "23" => "CREDIT_NOTE",
        "81" | "82" | "83" | "84" | "85" => "SALES_REPORT",
        "04" | "05" | "07" | "09" | "29" | "50" | "91" | "92" | "93" | "94" | "95" => "PROTOCOL",
        _ => "INVOICE",
    }
}

pub fn is_corrective_nap(code: &str) -> bool {
    matches!(code, "02" | "03" | "12" | "13" | "23")
}

/// Get document type name by code and entry type
pub fn get_doc_type_name(code: &str, entry_type: &str) -> Option<&'static str> {
    let types = if entry_type == "purchase" {
        PURCHASE_DOC_TYPES
    } else {
        SALES_DOC_TYPES
    };

    types.iter()
        .find(|e| e.code == code)
        .map(|e| e.name)
}

/// Get delivery code name by code
pub fn get_delivery_code_name(code: &str) -> Option<&'static str> {
    DELIVERY_CODES.iter()
        .find(|e| e.code == code)
        .map(|e| e.name)
}

/// Validate Bulgarian VAT number
pub fn validate_bulgarian_vat(vat_number: &str) -> bool {
    let vat = if vat_number.len() >= 2 &&
        (vat_number.starts_with("BG") || vat_number.starts_with("bg")) {
        &vat_number[2..]
    } else {
        vat_number
    };

    // Bulgarian VAT numbers are 9 or 10 digits
    if vat.len() < 9 || vat.len() > 10 {
        return false;
    }

    // Check all characters are digits
    vat.chars().all(|c| c.is_ascii_digit())
}

/// EU VAT country prefixes (EL = Greece in VIES, XI = Northern Ireland)
pub const EU_VAT_PREFIXES: &[&str] = &[
    "AT", "BE", "BG", "CY", "CZ", "DE", "DK", "EE", "EL", "ES",
    "FI", "FR", "GR", "HR", "HU", "IE", "IT", "LT", "LU", "LV", "MT",
    "NL", "PL", "PT", "RO", "SE", "SI", "SK", "XI",
];

/// Placeholder EIK for non-EU / unknown third-country counterparts (may repeat)
pub const GENERIC_EIK_9: &str = "999999999";
pub const GENERIC_EIK_13: &str = "9999999999999";

pub fn is_generic_eik(eik: &str) -> bool {
    let e = eik.trim();
    e == GENERIC_EIK_9 || e == GENERIC_EIK_13
}

/// Strip EU VAT letters (BG, IT, ES, …) so the remainder can be stored as EIK.
pub fn strip_eu_vat_prefix(value: &str) -> String {
    let v: String = value
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '.' && *c != '-')
        .collect::<String>()
        .to_uppercase();
    if v.len() > 2 {
        let prefix = &v[..2];
        if prefix.chars().all(|c| c.is_ascii_alphabetic()) && EU_VAT_PREFIXES.contains(&prefix) {
            return v[2..].to_string();
        }
    }
    v
}

/// Validate EU VAT number (basic format check)
pub fn validate_eu_vat(vat_number: &str) -> bool {
    if vat_number.len() < 4 {
        return false;
    }

    let prefix = vat_number[..2].to_uppercase();
    EU_VAT_PREFIXES.contains(&prefix.as_str())
}

/// Format VAT number for display
pub fn format_vat_number(vat_number: &str) -> String {
    if vat_number.is_empty() {
        return String::new();
    }

    // Add BG prefix if not present and looks like Bulgarian number
    if vat_number.len() >= 9 && vat_number.len() <= 10
        && vat_number.chars().all(|c| c.is_ascii_digit()) {
        return format!("BG{}", vat_number);
    }

    vat_number.to_string()
}

/// Month names in Bulgarian
pub const MONTH_NAMES: [&str; 12] = [
    "Януари", "Февруари", "Март", "Април", "Май", "Юни",
    "Юли", "Август", "Септември", "Октомври", "Ноември", "Декември"
];
