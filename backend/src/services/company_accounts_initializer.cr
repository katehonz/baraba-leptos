# Service for initializing company accounts from SAF-T standard chart of accounts
# При създаване на фирма се копира SAF-T сметкоплана с 1:1 mapping
class CompanyAccountsInitializer
  Log = ::Log.for("company_init")

  # Map SAF-T section codes to account types
  SECTION_TO_TYPE = {
    "1" => "equity",    # Сметки за капитал и заеми
    "2" => "asset",     # Сметки за дълготрайни активи
    "3" => "asset",     # Сметки за материални запаси
    "4" => "liability", # Сметки за разчети (обикновено пасиви, но може да са двустранни)
    "5" => "expense",   # Сметки за разходи
    "6" => "revenue",   # Сметки за приходи
    "7" => "expense",   # Сметки за разходи по икономически елементи
    "9" => "asset",     # Сметки за условни активи и пасиви / задбалансови
  }

  # More specific mappings based on group codes
  GROUP_TO_TYPE = {
    # Раздел 1 - Капитал
    "10" => "equity",    # Капитал
    "11" => "equity",    # Капиталови резерви
    "12" => "equity",    # Финансови резултати
    "15" => "liability", # Получени заеми (пасиви)

    # Раздел 2 - ДМА
    "20" => "asset", # ДМА
    "21" => "asset", # Нематериални активи
    "22" => "asset", # Дългосрочни инвестиции
    "24" => "asset", # Амортизация (контра-актив, но третираме като актив)

    # Раздел 3 - Материални запаси
    "30" => "asset", # Материали
    "31" => "asset", # Млади животни
    "32" => "asset", # Незавършено производство
    "33" => "asset", # Продукция
    "34" => "asset", # Стоки

    # Раздел 4 - Разчети
    "40" => "liability", # Доставчици
    "41" => "asset",     # Клиенти
    "42" => "liability", # Персонал
    "45" => "liability", # Данъчни разчети
    "46" => "asset",     # Вземания
    "49" => "liability", # Провизии

    # Раздел 5 - Парични средства
    "50" => "asset", # Каса
    "51" => "asset", # Банкови сметки
    "52" => "asset", # Краткосрочни инвестиции

    # Раздел 6 - Разходи
    "60" => "expense", # Разходи по икономически елементи
    "61" => "expense", # Разходи за дейността

    # Раздел 7 - Приходи
    "70" => "revenue", # Приходи от продажби
    "71" => "revenue", # Приходи от дейността
    "72" => "revenue", # Финансови приходи

    # Раздел 9 - Задбалансови
    "90" => "asset", # Условни активи
    "91" => "liability", # Условни пасиви
  }

  # Initialize accounts for a company from SAF-T standard accounts
  def self.initialize_accounts(company : Company) : NamedTuple(success: Bool, message: String, created: Int32)
    total_count = SaftAccountQuery.new.select_count
    if total_count == 0
      return {success: false, message: "SAF-T сметкоплан не е зареден. Моля заредете го първо.", created: 0}
    end

    created_count = 0
    errors = [] of String

    SaftAccountQuery.new.each do |saft_account|
      # Check if account already exists for this company
      existing = AccountQuery.new.company_id(company.id).code(saft_account.code).first?
      next if existing

      # Determine account type
      account_type = determine_account_type(saft_account)

      begin
        SaveAccount.create!(
          company_id: company.id,
          code: saft_account.code,
          name: saft_account.name,
          account_type: account_type,
          saft_account_id: saft_account.id,  # 1:1 mapping
          is_active: true
        )
        created_count += 1
      rescue ex
        errors << "#{saft_account.code}: #{ex.message}"
        Log.error { "Failed to create account #{saft_account.code}: #{ex.message}" }
      end
    end

    if errors.empty?
      {success: true, message: "Създадени #{created_count} сметки от SAF-T сметкоплана", created: created_count}
    else
      {success: true, message: "Създадени #{created_count} сметки. Грешки: #{errors.size}", created: created_count}
    end
  end

  # Determine account type based on SAF-T account code and section
  private def self.determine_account_type(saft_account : SaftAccount) : String
    code = saft_account.code

    # First try specific group mapping (first 2 digits)
    if code.size >= 2
      group = code[0, 2]
      if type = GROUP_TO_TYPE[group]?
        return type
      end
    end

    # Fall back to section mapping (first digit)
    if code.size >= 1
      section = code[0, 1]
      if type = SECTION_TO_TYPE[section]?
        return type
      end
    end

    # Default to asset if we can't determine
    "asset"
  end
end
