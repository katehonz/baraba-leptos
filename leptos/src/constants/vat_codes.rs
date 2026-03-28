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

/// Document types for purchases (Покупки)
pub const PURCHASE_DOC_TYPES: &[CodeEntry] = &[
    CodeEntry::new("01", "Фактура"),
    CodeEntry::new("02", "Дебитно известие"),
    CodeEntry::new("03", "Кредитно известие"),
    CodeEntry::new("05", "Регистър стоки до поискване"),
    CodeEntry::new("07", "Митническа декларация"),
    CodeEntry::new("09", "Протокол"),
    CodeEntry::new("11", "Фактура - касова отчетност"),
    CodeEntry::new("12", "ДИ - касова отчетност"),
    CodeEntry::new("13", "КИ - касова отчетност"),
    CodeEntry::new("23", "КИ по чл.126б"),
    CodeEntry::new("91", "Протокол чл.151в ал.3"),
    CodeEntry::new("92", "Протокол чл.151г ал.8"),
    CodeEntry::new("93", "Протокол чл.151в ал.7 (без режим)"),
    CodeEntry::new("94", "Протокол чл.151в ал.7 (с режим)"),
];

/// Document types for sales (Продажби)
pub const SALES_DOC_TYPES: &[CodeEntry] = &[
    CodeEntry::new("01", "Фактура"),
    CodeEntry::new("02", "Дебитно известие"),
    CodeEntry::new("03", "Кредитно известие"),
    CodeEntry::new("04", "Регистър стоки до поискване"),
    CodeEntry::new("09", "Протокол"),
    CodeEntry::new("11", "Фактура - касова отчетност"),
    CodeEntry::new("12", "ДИ - касова отчетност"),
    CodeEntry::new("13", "КИ - касова отчетност"),
    CodeEntry::new("29", "КИ по чл.126б"),
    CodeEntry::new("50", "Известие за продажба"),
    CodeEntry::new("81", "Отчет за извършени продажби"),
    CodeEntry::new("82", "Отчет продажби чл.119"),
    CodeEntry::new("83", "Отчет за безвъзмездни доставки"),
    CodeEntry::new("84", "Отчет за корекции на ДДС"),
    CodeEntry::new("85", "Отчет доставки чл.163а"),
    CodeEntry::new("95", "Протокол по чл.117 ал.4"),
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

/// Validate EU VAT number (basic format check)
pub fn validate_eu_vat(vat_number: &str) -> bool {
    if vat_number.len() < 4 {
        return false;
    }

    let prefix = vat_number[..2].to_uppercase();
    let eu_countries = [
        "AT", "BE", "BG", "CY", "CZ", "DE", "DK", "EE", "EL", "ES",
        "FI", "FR", "HR", "HU", "IE", "IT", "LT", "LU", "LV", "MT",
        "NL", "PL", "PT", "RO", "SE", "SI", "SK", "XI"
    ];

    eu_countries.contains(&prefix.as_str())
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
