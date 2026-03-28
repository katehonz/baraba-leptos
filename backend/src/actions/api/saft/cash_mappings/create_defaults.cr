# POST /api/companies/:company_id/saft/cash_mappings/create_defaults
class Api::Saft::CashMappings::CreateDefaults < ApiAction
  include Api::Auth::SkipRequireAuthToken

  # Default mappings for Bulgarian accounting
  DEFAULT_CASH_MAPPINGS = [
    # 10 = Плащане в брой - приход (Cash receipt)
    {code: "10", dt: "501", kt: "411", desc: "Каса от клиенти"},
    {code: "10", dt: "501", kt: "709", desc: "Каса други приходи"},
    {code: "10", dt: "501", kt: "721", desc: "Каса лихви"},

    # 11 = Плащане в брой - разход (Cash payment)
    {code: "11", dt: "401", kt: "501", desc: "Каса към доставчици"},
    {code: "11", dt: "602", kt: "501", desc: "Каса външни услуги"},
    {code: "11", dt: "604", kt: "501", desc: "Каса заплати"},
    {code: "11", dt: "609", kt: "501", desc: "Каса други разходи"},

    # 42 = Банков превод - приход (Bank transfer receipt)
    {code: "42", dt: "503", kt: "411", desc: "Банка от клиенти"},
    {code: "42", dt: "503", kt: "709", desc: "Банка други приходи"},
    {code: "42", dt: "503", kt: "721", desc: "Банка лихви"},
    {code: "42", dt: "503", kt: "752", desc: "Банка фин. приходи"},

    # 43 = Банков превод - разход (Bank transfer payment)
    {code: "43", dt: "401", kt: "503", desc: "Банка към доставчици"},
    {code: "43", dt: "602", kt: "503", desc: "Банка външни услуги"},
    {code: "43", dt: "604", kt: "503", desc: "Банка заплати"},
    {code: "43", dt: "609", kt: "503", desc: "Банка други разходи"},
    {code: "43", dt: "621", kt: "503", desc: "Банка лихви по кредити"},
    {code: "43", dt: "629", kt: "503", desc: "Банка такси"},

    # 48 = Плащане с карта - приход (Card payment receipt)
    {code: "48", dt: "503", kt: "411", desc: "Карта от клиенти"},

    # 49 = Плащане с карта - разход (Card payment)
    {code: "49", dt: "401", kt: "503", desc: "Карта към доставчици"},

    # 80 = Вътрешен трансфер (Internal transfer)
    {code: "80", dt: "503", kt: "501", desc: "Внасяне каса в банка"},
    {code: "80", dt: "501", kt: "503", desc: "Теглене от банка в каса"},
    {code: "80", dt: "503", kt: "503", desc: "Между банкови сметки"},

    # 90 = Валутна разлика - положителна (FX gain)
    {code: "90", dt: "503", kt: "724", desc: "Положителна валутна разлика"},

    # 91 = Валутна разлика - отрицателна (FX loss)
    {code: "91", dt: "624", kt: "503", desc: "Отрицателна валутна разлика"},

    # 97 = Прихващане (Offset)
    {code: "97", dt: "401", kt: "411", desc: "Прихващане доставчик-клиент"},

    # 99 = Подотчетни лица (Accountable persons)
    {code: "99", dt: "422", kt: "501", desc: "Служебен аванс"},
    {code: "99", dt: "501", kt: "422", desc: "Отчитане служебен аванс"},
  ]

  post "/api/companies/:company_id/saft/cash_mappings/create_defaults" do
    # Check if company already has mappings
    existing = SaftCashAccountMappingQuery.new.company_id(company_id).select_count

    if existing > 0
      return json({success: false, error: "Company already has #{existing} mappings. Delete them first."})
    end

    DEFAULT_CASH_MAPPINGS.each do |mapping|
      SaveSaftCashAccountMapping.create!(
        company_id: company_id.to_i64,
        cash_movement_type: mapping[:code],
        debit_account: mapping[:dt],
        credit_account: mapping[:kt],
        description: mapping[:desc],
        is_active: true
      )
    end

    count = SaftCashAccountMappingQuery.new.company_id(company_id).select_count

    json({success: true, message: "Created #{count} default mappings"})
  rescue ex
    json({success: false, error: ex.message})
  end
end
