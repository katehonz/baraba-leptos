/// Bulgarian amount-in-words (лева/евро), half-up stotinki, feminine for thousands/stotinki.
pub fn convert(amount: f64, currency_code: &str) -> String {
    let negative = amount < 0.0;
    let cents = (amount.abs() * 100.0 + 1e-9).round() as i64;
    let leva = cents / 100;
    let stotinki = (cents % 100) as i32;
    let mut words = if leva == 0 {
        "нула".to_string()
    } else {
        convert_long(leva)
    };
    let eur = currency_code.eq_ignore_ascii_case("EUR");
    words.push(' ');
    if eur {
        words.push_str("евро");
    } else {
        words.push_str(if leva == 1 { "лев" } else { "лева" });
    }
    if stotinki > 0 {
        words.push_str(" и ");
        words.push_str(&convert_small(stotinki, true));
        words.push(' ');
        if eur {
            words.push_str(if stotinki == 1 { "цент" } else { "цента" });
        } else {
            words.push_str(if stotinki == 1 { "стотинка" } else { "стотинки" });
        }
    }
    let trimmed = words.trim().to_string();
    if trimmed.is_empty() {
        return String::new();
    }
    if negative {
        format!("Минус {}", trimmed)
    } else {
        let mut chars = trimmed.chars();
        match chars.next() {
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    }
}

fn convert_long(mut n: i64) -> String {
    if n < 1000 {
        return convert_small(n as i32, false);
    }
    let mut parts: Vec<String> = Vec::new();
    if n >= 1_000_000 {
        let millions = (n / 1_000_000) as i32;
        if millions == 1 {
            parts.push("един милион".into());
        } else {
            parts.push(format!("{} милиона", convert_small(millions, false)));
        }
        n %= 1_000_000;
    }
    if n >= 1000 {
        let thousands = (n / 1000) as i32;
        if thousands == 1 {
            parts.push("хиляда".into());
        } else {
            parts.push(format!("{} хиляди", convert_small(thousands, true)));
        }
        n %= 1000;
    }
    if n > 0 {
        let rest = convert_small(n as i32, false);
        if parts.is_empty() {
            parts.push(rest);
        } else {
            parts.push(format!("и {}", rest));
        }
    }
    parts.join(" ")
}

const UNITS: [&str; 10] = ["", "един", "два", "три", "четири", "пет", "шест", "седем", "осем", "девет"];
const UNITS_FEM: [&str; 10] = ["", "една", "две", "три", "четири", "пет", "шест", "седем", "осем", "девет"];
const TEENS: [&str; 10] = [
    "десет", "единадесет", "дванадесет", "тринадесет", "четиринадесет",
    "петнадесет", "шестнадесет", "седемнадесет", "осемнадесет", "деветнадесет",
];
const TENS: [&str; 10] = ["", "", "двадесет", "тридесет", "четиридесет", "петдесет", "шестдесет", "седемдесет", "осемдесет", "деветдесет"];
const HUNDREDS: [&str; 10] = ["", "сто", "двеста", "триста", "четиристотин", "петстотин", "шестстотин", "седемстотин", "осемстотин", "деветстотин"];

fn convert_small(n: i32, feminine: bool) -> String {
    let h = n / 100;
    let rem = n % 100;
    let mut s = String::new();
    if h > 0 {
        s.push_str(HUNDREDS[h as usize]);
        s.push(' ');
    }
    if rem > 0 {
        if h > 0 && (rem < 20 || rem % 10 == 0) {
            s.push_str("и ");
        }
        if rem < 10 {
            s.push_str(if feminine { UNITS_FEM[rem as usize] } else { UNITS[rem as usize] });
        } else if rem < 20 {
            s.push_str(TEENS[(rem - 10) as usize]);
        } else {
            s.push_str(TENS[(rem / 10) as usize]);
            let u = rem % 10;
            if u > 0 {
                s.push_str(" и ");
                s.push_str(if feminine { UNITS_FEM[u as usize] } else { UNITS[u as usize] });
            }
        }
    }
    s.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_lev() {
        assert_eq!(convert(1.0, "BGN"), "Един лев");
    }
    #[test]
    fn two_leva() {
        assert!(convert(2.0, "BGN").starts_with("Два лева"));
    }
    #[test]
    fn euro_cents() {
        let s = convert(1.05, "EUR");
        assert!(s.contains("евро"));
        assert!(s.contains("цент"));
    }
    #[test]
    fn fixtures() {
        assert_eq!(convert(0.0, "BGN"), "Нула лева");
        assert_eq!(convert(0.01, "BGN"), "Нула лева и една стотинка");
        assert_eq!(convert(21.0, "BGN"), "Двадесет и един лева");
        assert_eq!(convert(100.0, "BGN"), "Сто лева");
        assert_eq!(convert(101.0, "BGN"), "Сто и един лева");
        assert_eq!(convert(1000.0, "BGN"), "Хиляда лева");
        assert_eq!(convert(1001.0, "BGN"), "Хиляда и един лева");
        assert_eq!(convert(1_000_000.0, "BGN"), "Един милион лева");
        assert_eq!(convert(120.0, "EUR"), "Сто и двадесет евро");
        assert_eq!(convert(120.0, "BGN"), "Сто и двадесет лева");
        assert_eq!(convert(-120.0, "BGN"), "Минус сто и двадесет лева");
        assert_eq!(convert(1.005, "BGN"), "Един лев и една стотинка");
    }
}
