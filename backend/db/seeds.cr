# Seed data for reference tables

# Helper to save models with error handling
def seed_model(model, action_name : String)
  if model.save
    puts "✓ #{action_name}"
  else
    puts "✗ #{action_name}: #{model.errors.inspect}"
  end
end

puts "Seeding reference data..."
puts "=" * 50

# ============================================
# ISO Countries (ISO 3166-1)
# ============================================
puts "\n📍 Seeding ISO Countries..."

countries = [
  {"BG", "BGR", "Bulgaria", "България", "100", true},
  {"DE", "DEU", "Germany", "Германия", "276", true},
  {"FR", "FRA", "France", "Франция", "250", true},
  {"IT", "ITA", "Italy", "Италия", "380", true},
  {"ES", "ESP", "Spain", "Испания", "724", true},
  {"PL", "POL", "Poland", "Полша", "616", true},
  {"RO", "ROU", "Romania", "Румъния", "642", true},
  {"GR", "GRC", "Greece", "Гърция", "300", true},
  {"NL", "NLD", "Netherlands", "Холандия", "528", true},
  {"BE", "BEL", "Belgium", "Белгия", "056", true},
  {"AT", "AUT", "Austria", "Австрия", "040", true},
  {"CZ", "CZE", "Czech Republic", "Чехия", "203", true},
  {"HU", "HUN", "Hungary", "Унгария", "348", true},
  {"PT", "PRT", "Portugal", "Португалия", "620", true},
  {"SE", "SWE", "Sweden", "Швеция", "752", true},
  {"DK", "DNK", "Denmark", "Дания", "208", true},
  {"FI", "FIN", "Finland", "Финландия", "246", true},
  {"IE", "IRL", "Ireland", "Ирландия", "372", true},
  {"LU", "LUX", "Luxembourg", "Люксембург", "442", true},
  {"US", "USA", "United States", "САЩ", "840", false},
  {"GB", "GBR", "United Kingdom", "Великобритания", "826", false},
  {"CH", "CHE", "Switzerland", "Швейцария", "756", false},
  {"NO", "NOR", "Norway", "Норвегия", "578", false},
  {"UA", "UKR", "Ukraine", "Украйна", "804", false},
  {"TR", "TUR", "Turkey", "Турция", "792", false},
  {"RU", "RUS", "Russia", "Русия", "643", false},
  {"CN", "CHN", "China", "Китай", "156", false},
  {"JP", "JPN", "Japan", "Япония", "392", false},
  {"IN", "IND", "India", "Индия", "356", false},
  {"BR", "BRA", "Brazil", "Бразилия", "076", false},
  {"AU", "AUS", "Australia", "Австралия", "036", false},
  {"CA", "CAN", "Canada", "Канада", "124", false},
  {"ZA", "ZAF", "South Africa", "Южна Африка", "710", false},
]

countries.each do |country|
  alpha2, alpha3, name, name_bg, numeric, is_eu = country
  unless IsoCountryQuery.new.alpha2(alpha2).first?
    iso_country = IsoCountry.new(
      alpha2: alpha2,
      alpha3: alpha3,
      name: name,
      name_bg: name_bg,
      numeric: numeric,
      is_eu_member: is_eu
    )
    seed_model(iso_country, "Country: #{name}")
  end
end

# ============================================
# ISO Currencies (ISO 4217)
# ============================================
puts "\n💰 Seeding ISO Currencies..."

currencies = [
  {"BGN", "Bulgarian Lev", "Български лев", "975", "лв", 2},
  {"EUR", "Euro", "Евро", "978", "€", 2},
  {"USD", "US Dollar", "Щатски долар", "840", "$", 2},
  {"GBP", "British Pound", "Британски паунд", "826", "£", 2},
  {"CHF", "Swiss Franc", "Швейцарски франк", "756", "Fr", 2},
  {"JPY", "Japanese Yen", "Японска йена", "392", "¥", 0},
  {"CNY", "Chinese Yuan", "Китайски юан", "156", "¥", 2},
  {"RUB", "Russian Ruble", "Руски рубъл", "643", "₽", 2},
  {"PLN", "Polish Zloty", "Полска злота", "985", "zł", 2},
  {"CZK", "Czech Koruna", "Чешка крона", "203", "Kč", 2},
  {"HUF", "Hungarian Forint", "Унгарски форинт", "348", "Ft", 2},
  {"RON", "Romanian Leu", "Румънска лея", "946", "lei", 2},
  {"SEK", "Swedish Krona", "Шведска крона", "752", "kr", 2},
  {"DKK", "Danish Krone", "Датска крона", "208", "kr", 2},
  {"NOK", "Norwegian Krone", "Норвежка крона", "578", "kr", 2},
  {"TRY", "Turkish Lira", "Турска лира", "949", "₺", 2},
  {"AUD", "Australian Dollar", "Австралийски долар", "036", "A$", 2},
  {"CAD", "Canadian Dollar", "Канадски долар", "124", "C$", 2},
  {"INR", "Indian Rupee", "Индийска рупия", "356", "₹", 2},
  {"BRL", "Brazilian Real", "Бразилски реал", "986", "R$", 2},
]

currencies.each do |currency|
  code, name, name_bg, numeric, symbol, decimal_places = currency
  unless IsoCurrencyQuery.new.code(code).first?
    iso_currency = IsoCurrency.new(
      code: code,
      name: name,
      name_bg: name_bg,
      numeric: numeric,
      symbol: symbol,
      decimal_places: decimal_places
    )
    seed_model(iso_currency, "Currency: #{code} - #{name}")
  end
end

# ============================================
# SAF-T Invoice Types (Bulgaria)
# ============================================
puts "\n📄 Seeding SAF-T Invoice Types..."

invoice_types = [
  {"FT", "Factura", "Фактура за доставени стоки и оказани услуги на територията на страната от регистрирано лице по ЗДДС"},
  {"FK", "Kreditno izvestie", "Кредитно известие за фактурирани доставки на стоки и услуги"},
  {"DT", "Dogovor za arenda", "Договор за аренда"},
  {"KT", "Kreditno izvestie", "Кредитно известие"},
  {"FP", "Proteben faktura", "Протебена фактура"},
  {"PK", "Kreditno izvestie", "Претоварено кредитно известие"},
  {"PP", "Proteben protokol", "Претоварен протокол"},
  {"DK", "Dogovor za kredit", "Договор за кредит"},
  {"VD", "Vneshnotorgovska dokumentatsiya", "Външнотърговска документация"},
  {"VF", "Faktura za eksport", "Фактура за експорт"},
  {"VI", "Faktura za inport", "Фактура за внос"},
]

invoice_types.each do |invoice_type|
  code, name, description = invoice_type
  unless SaftInvoiceTypeQuery.new.code(code).first?
    saft_invoice_type = SaftInvoiceType.new(
      code: code,
      name: name,
      description: description
    )
    seed_model(saft_invoice_type, "SAFT Invoice Type: #{code} - #{name}")
  end
end

# ============================================
# SAF-T Payment Methods (Bulgaria)
# ============================================
puts "\n💳 Seeding SAF-T Payment Methods..."

payment_methods = [
  {"NU", "Nalichni", "Cash payment", "Плащане в брой"},
  {"BA", "Bankov prevod", "Bank transfer", "Банков превод"},
  {"KA", "Karta", "Card payment", "Плащане с карта"},
  {"CE", "Check", "Check payment", "Плащане с чек"},
  {"OU", "Other", "Other payment method", "Друг метод на плащане"},
  {"MP", "Mobilno platane", "Mobile payment", "Мобилно плащане"},
  {"EP", "E-pay", "Electronic payment", "Електронно плащане"},
  {"PF", "PayPal", "PayPal", "PayPal"},
  {"BB", "Banka po zastrahovane", "Bank guarantee", "Банков депозит"},
  {"PR", "Pretedatel", "Offset", "Претеждане"},
]

payment_methods.each do |payment_method|
  method_code, mechanism_code, name, description = payment_method
  unless SaftPaymentMethodQuery.new.method_code(method_code).first?
    saft_payment_method = SaftPaymentMethod.new(
      method_code: method_code,
      mechanism_code: mechanism_code,
      name: name,
      description: description
    )
    seed_model(saft_payment_method, "SAFT Payment Method: #{method_code} - #{name}")
  end
end

# ============================================
# SAF-T Tax Types (Bulgaria)
# ============================================
puts "\n📊 Seeding SAF-T Tax Types..."

tax_types = [
  {"IVA", "DDS", "Данък върху добавената стойност"},
  {"AD", "Aktsiz", "Акциз"},
  {"VD", "Vnatreshno darzhavno", "Вътрешен данък върху потреблението"},
  {"PD", "Pridavka darzhava", "Придавка държавна"},
  {"MD", "Mestni darzhavi", "Местни данъци"},
  {"TD", "Taksa dop", "Такса"},
  {"GL", "Globa", "Глоба"},
  {"LS", "Lichna", "Лична санкция"},
  {"DD", "Drugo darzhavno", "Друго данъчно задължение"},
  {"ND", "Ne podlezha", "Не подлежи на облагане"},
]

tax_types.each do |tax_type|
  code, name, description = tax_type
  unless SaftTaxTypeQuery.new.code(code).first?
    saft_tax_type = SaftTaxType.new(
      code: code,
      name: name,
      description: description
    )
    seed_model(saft_tax_type, "SAFT Tax Type: #{code} - #{name}")
  end
end

# ============================================
# VAT Rates (Bulgaria)
# ============================================
puts "\n📈 Seeding VAT Rates for Bulgaria..."

vat_rates = [
  {"Standard", 20.0, "S", "IVA", "NOR", Time.utc(2007, 1, 1), nil, true},
  {"Reduced", 9.0, "R", "IVA", "RED", Time.utc(2011, 4, 1), nil, true},
  {"Super Reduced", 0.0, "Z", "IVA", "ISE", Time.utc(2007, 1, 1), nil, true},
  {"Zero Rated", 0.0, "E", "IVA", "ISE", Time.utc(2007, 1, 1), nil, true},
  {"Exempt", 0.0, "M", "IVA", "ISE", Time.utc(2007, 1, 1), nil, true},
  {"Outside Scope", 0.0, "O", "IVA", "NS", Time.utc(2007, 1, 1), nil, true},
]

vat_rates.each do |vat_rate|
  name, percentage, code, saft_tax_type, saft_tax_code, effective_from, effective_to, is_active = vat_rate
  unless VatRateQuery.new.code(code).first?
    vat = VatRate.new(
      name: name,
      percentage: percentage,
      code: code,
      saft_tax_type: saft_tax_type,
      saft_tax_code: saft_tax_code,
      effective_from: effective_from,
      effective_to: effective_to,
      is_active: is_active
    )
    seed_model(vat, "VAT Rate: #{code} - #{name} (#{percentage}%)")
  end
end

# ============================================
# Test Company
# ============================================
puts "\n🏢 Seeding Test Company..."

# Demo Company - fictitious data for testing purposes only
unless CompanyQuery.new.first?
  company = Company.new(
    name: "Демо ООД",
    legal_name: "Демо Компания ООД",
    eik: "000000000",
    vat_number: "BG000000000",
    address: "София, ул. Примерна 1",
    city: "София",
    postal_code: "1000",
    country: "BG",
    email: "demo@example.com",
    phone: "+359 2 000 0000",
    default_currency: "BGN",
    fiscal_year_start: 1
  )
  seed_model(company, "Demo Company: Демо ООД (fictitious data)")
end

# ============================================
# Summary
# ============================================
puts "\n" + "=" * 50
puts "✅ Seeding completed!"
puts "=" * 50
puts "\n📊 Summary:"
puts "   Countries: #{IsoCountryQuery.select_count}"
puts "   Currencies: #{IsoCurrencyQuery.select_count}"
puts "   SAFT Invoice Types: #{SaftInvoiceTypeQuery.select_count}"
puts "   SAFT Payment Methods: #{SaftPaymentMethodQuery.select_count}"
puts "   SAFT Tax Types: #{SaftTaxTypeQuery.select_count}"
puts "   VAT Rates: #{VatRateQuery.select_count}"
puts "\n"
