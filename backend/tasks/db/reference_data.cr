# Seed reference data for application
#
# This task adds ISO countries, currencies, SAFT types, and VAT rates
# required for accounting system to work properly.

class Db::Seed::ReferenceData < LuckyTask::Task
  summary "Add reference data (countries, currencies, SAFT types, VAT rates)"

  def call
    puts "Seeding reference data..."
    puts "=" * 50

    seed_countries
    seed_currencies
    seed_saft_invoice_types
    seed_saft_payment_methods
    seed_saft_tax_types
    seed_vat_rates
    seed_roles
    seed_system_settings
    seed_test_company
    seed_admin_user

    puts "\n" + "=" * 50
    puts "✅ Seeding completed!"
    puts "=" * 50
    puts "\n📊 Summary:"
    puts "   Countries: #{IsoCountryQuery.new.select_count}"
    puts "   Currencies: #{IsoCurrencyQuery.new.select_count}"
    puts "   SAFT Invoice Types: #{SaftInvoiceTypeQuery.new.select_count}"
    puts "   SAFT Payment Methods: #{SaftPaymentMethodQuery.new.select_count}"
    puts "   SAFT Tax Types: #{SaftTaxTypeQuery.new.select_count}"
    puts "   VAT Rates: #{VatRateQuery.new.select_count}"
    puts "   Roles: #{RoleQuery.new.select_count}"
    puts "   System Settings: #{SystemSettingQuery.new.select_count}"
    puts "   Companies: #{CompanyQuery.new.select_count}"
    puts "   Users: #{UserQuery.new.select_count}"
    puts "\n"
  end

  private def seed_countries
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
        AppDatabase.exec(
          "INSERT INTO iso_countries (alpha2, alpha3, name, name_bg, numeric, is_eu_member, created_at, updated_at) " +
          "VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW()) ON CONFLICT (alpha2) DO NOTHING",
          [alpha2, alpha3, name, name_bg, numeric, is_eu]
        )
        puts "✓ Country: #{name}"
      end
    end
  end

  private def seed_currencies
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
        AppDatabase.exec(
          "INSERT INTO iso_currencies (code, name, name_bg, numeric, symbol, decimal_places, created_at, updated_at) " +
          "VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW()) ON CONFLICT (code) DO NOTHING",
          [code, name, name_bg, numeric, symbol, decimal_places]
        )
        puts "✓ Currency: #{code} - #{name}"
      end
    end
  end

  private def seed_saft_invoice_types
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
        AppDatabase.exec(
          "INSERT INTO saft_invoice_types (code, name, description, created_at, updated_at) " +
          "VALUES ($1, $2, $3, NOW(), NOW()) ON CONFLICT (code) DO NOTHING",
          [code, name, description]
        )
        puts "✓ SAFT Invoice Type: #{code} - #{name}"
      end
    end
  end

  private def seed_saft_payment_methods
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
        AppDatabase.exec(
          "INSERT INTO saft_payment_methods (method_code, mechanism_code, name, description, created_at, updated_at) " +
          "VALUES ($1, $2, $3, $4, NOW(), NOW()) ON CONFLICT (method_code) DO NOTHING",
          [method_code, mechanism_code, name, description]
        )
        puts "✓ SAFT Payment Method: #{method_code} - #{name}"
      end
    end
  end

  private def seed_saft_tax_types
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
        AppDatabase.exec(
          "INSERT INTO saft_tax_types (code, name, description, created_at, updated_at) " +
          "VALUES ($1, $2, $3, NOW(), NOW()) ON CONFLICT (code) DO NOTHING",
          [code, name, description]
        )
        puts "✓ SAFT Tax Type: #{code} - #{name}"
      end
    end
  end

  private def seed_vat_rates
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
        AppDatabase.exec(
          "INSERT INTO vat_rates (name, percentage, code, saft_tax_type, saft_auth_code, effective_from, effective_to, is_active, created_at, updated_at) " +
          "VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW()) ON CONFLICT (code) DO NOTHING",
          [name, percentage, code, saft_tax_type, saft_tax_code, effective_from, effective_to, is_active]
        )
        puts "✓ VAT Rate: #{code} - #{name} (#{percentage}%)"
      end
    end
  end

  private def seed_test_company
    puts "\n🏢 Seeding Test Company..."

    unless CompanyQuery.new.first?
      AppDatabase.exec(
        "INSERT INTO companies (name, legal_name, eik, vat_number, address, city, postal_code, country, email, phone, default_currency, fiscal_year_start, created_at, updated_at) " +
        "VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW())",
        ["Демо ООД", "Демо Компания ООД", "000000000", "BG000000000", "София, ул. Примерна 1", "София", "1000", "BG", "demo@example.com", "+359 2 000 0000", "BGN", 1]
      )
      puts "✓ Demo Company: Демо ООД (fictitious data)"
    end
  end

  private def seed_roles
    puts "\n🔐 Seeding Roles..."

    # Using proper permission constants from Permission module
    super_admin_permissions = [Permission::ALL].to_json
    admin_permissions = (Permission::ADMIN_ALL + Permission::USER_ALL + Permission::COMPANY_ALL + Permission::ROLE_ALL + [Permission::SETTINGS_READ, Permission::SETTINGS_UPDATE] + Permission::REPORT_ALL).to_json
    accountant_permissions = (Permission::ACCOUNTING_ALL + Permission::INVOICE_ALL + Permission::PAYMENT_ALL + Permission::VAT_ALL + Permission::DOCUMENT_ALL + [Permission::REPORT_READ, Permission::SETTINGS_READ]).to_json
    viewer_permissions = Permission::READ_ONLY.to_json

    roles = [
      {"super_admin", "Супер Администратор", super_admin_permissions},
      {"admin", "Администратор", admin_permissions},
      {"accountant", "Счетоводител", accountant_permissions},
      {"viewer", "Наблюдател", viewer_permissions},
    ]

    roles.each do |role|
      name, description, permissions = role
      unless RoleQuery.new.name(name).first?
        AppDatabase.exec(
          "INSERT INTO roles (name, description, permissions, created_at, updated_at) " +
          "VALUES ($1, $2, $3, NOW(), NOW()) ON CONFLICT DO NOTHING",
          [name, description, permissions]
        )
        puts "✓ Role: #{name} - #{description}"
      end
    end
  end

  private def seed_system_settings
    puts "\n⚙️ Seeding System Settings..."

    settings = [
      {
        "smtp",
        {
          "host"       => "",
          "port"       => 587,
          "username"   => "",
          "password"   => "",
          "from_email" => "",
          "from_name"  => "Baraba",
          "use_tls"    => true,
          "enabled"    => false,
        }.to_json,
        "SMTP настройки за изходяща поща",
      },
      {
        "app",
        {
          "name"                 => "Baraba",
          "url"                  => "http://localhost:3000",
          "default_language"     => "bg",
          "registration_enabled" => true,
          "version"              => "1.0.0",
        }.to_json,
        "Общи настройки на приложението",
      },
      {
        "security",
        {
          "session_timeout_minutes" => 60,
          "max_login_attempts"      => 5,
          "lockout_duration_minutes" => 15,
          "password_min_length"     => 8,
          "require_2fa"             => false,
        }.to_json,
        "Настройки за сигурност",
      },
    ]

    settings.each do |setting|
      key, value, description = setting
      unless SystemSettingQuery.new.key(key).first?
        AppDatabase.exec(
          "INSERT INTO system_settings (key, value, description, created_at, updated_at) " +
          "VALUES ($1, $2, $3, NOW(), NOW()) ON CONFLICT (key) DO NOTHING",
          [key, value, description]
        )
        puts "✓ System Setting: #{key}"
      end
    end
  end

  private def seed_admin_user
    puts "\n👤 Seeding Super Admin User..."

    admin_email = ENV["ADMIN_EMAIL"]? || "admin@example.com"
    admin_password = ENV["ADMIN_PASSWORD"]?

    unless UserQuery.new.email(admin_email).first?
      if admin_password.nil? || admin_password.empty?
        # Generate a random password and print it once
        admin_password = Random::Secure.hex(16)
        puts "⚠️  Generated admin password: #{admin_password}"
        puts "⚠️  SAVE THIS PASSWORD! It will not be shown again."
      end

      # Create admin user with encrypted password and super admin flag
      encrypted = Authentic.generate_encrypted_password(admin_password)

      AppDatabase.exec(
        "INSERT INTO users (email, encrypted_password, is_super_admin, email_verified_at, created_at, updated_at) " +
        "VALUES ($1, $2, $3, NOW(), NOW(), NOW())",
        [admin_email, encrypted, true]
      )
      puts "✓ Admin User: #{admin_email} (Super Admin)"

      # Link admin to first company with super_admin role
      admin_user = UserQuery.new.email(admin_email).first?
      company = CompanyQuery.new.first?
      super_admin_role = RoleQuery.new.name("super_admin").first?

      if admin_user && company && super_admin_role
        AppDatabase.exec(
          "INSERT INTO user_company_roles (user_id, company_id, role_id, created_at, updated_at) " +
          "VALUES ($1, $2, $3, NOW(), NOW()) ON CONFLICT DO NOTHING",
          [admin_user.id, company.id, super_admin_role.id]
        )
        puts "✓ Linked admin to company with super_admin role"

        # Set admin as company owner
        AppDatabase.exec(
          "UPDATE companies SET owner_id = $1 WHERE id = $2",
          [admin_user.id, company.id]
        )
        puts "✓ Set admin as company owner"
      end
    else
      puts "• Admin user already exists (skipping)"
    end
  end
end
