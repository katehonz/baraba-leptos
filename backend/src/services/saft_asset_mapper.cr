# Service for SAF-T Asset Account Mappings
# Стандартни кореспонденции за движения на ДМА
class SaftAssetMapper
  # Default mappings for Bulgarian accounting
  # Based on Bulgarian Chart of Accounts (Национален сметкоплан)
  DEFAULT_MAPPINGS = [
    # 10 - Придобиване (Acquisition)
    {code: "10", dt: "203", kt: "401", desc: "Придобиване на машини от доставчик"},
    {code: "10", dt: "204", kt: "401", desc: "Придобиване на съоръжения от доставчик"},
    {code: "10", dt: "205", kt: "401", desc: "Придобиване на транспортни средства"},
    {code: "10", dt: "206", kt: "401", desc: "Придобиване на стопански инвентар"},
    {code: "10", dt: "207", kt: "401", desc: "Придобиване на компютърна техника"},
    {code: "10", dt: "201", kt: "401", desc: "Придобиване на земи"},
    {code: "10", dt: "202", kt: "401", desc: "Придобиване на сгради"},
    {code: "10", dt: "20*", kt: "613", desc: "Придобиване със собствени сили"},

    # 20 - Продажба (Sale)
    {code: "20", dt: "411", kt: "705", desc: "Продажба на ДМА - приход"},
    {code: "20", dt: "241", kt: "20*", desc: "Отписване на амортизация при продажба"},
    {code: "20", dt: "705", kt: "20*", desc: "Отписване на балансова стойност"},

    # 30 - Амортизация (Depreciation)
    {code: "30", dt: "603", kt: "241", desc: "Начисляване на амортизация на машини"},
    {code: "30", dt: "603", kt: "242", desc: "Начисляване на амортизация на сгради"},
    {code: "30", dt: "603", kt: "24*", desc: "Начисляване на амортизация (общо)"},

    # 40 - Вътрешен трансфер (Internal Transfer)
    {code: "40", dt: "20*", kt: "20*", desc: "Вътрешен трансфер между поделения"},

    # 50 - Брак (Scrap/Write-off)
    {code: "50", dt: "241", kt: "20*", desc: "Отписване на амортизация при брак"},
    {code: "50", dt: "609", kt: "20*", desc: "Отписване на остатъчна стойност при брак"},

    # 60 - Преоценка отрицателна (Negative Revaluation)
    {code: "60", dt: "608", kt: "20*", desc: "Отрицателна преоценка на ДМА"},

    # 70 - Преоценка положителна (Positive Revaluation)
    {code: "70", dt: "20*", kt: "112", desc: "Положителна преоценка на ДМА"},

    # 80 - Излишък при инвентаризация (Inventory Surplus)
    {code: "80", dt: "20*", kt: "709", desc: "Установен излишък на ДМА"},

    # 90 - Липса при инвентаризация (Inventory Shortage)
    {code: "90", dt: "609", kt: "20*", desc: "Установена липса на ДМА"},
    {code: "90", dt: "241", kt: "20*", desc: "Отписване на амортизация при липса"},

    # 100 - Обезценка (Impairment)
    {code: "100", dt: "608", kt: "20*", desc: "Обезценка на ДМА"},

    # 110 - Сторно на обезценка (Reversal of Impairment)
    {code: "110", dt: "20*", kt: "709", desc: "Възстановяване на обезценка"},

    # 120 - Безвъзмездно предоставени (Donated/Granted)
    {code: "120", dt: "609", kt: "20*", desc: "Безвъзмездно предоставени ДМА"},
    {code: "120", dt: "241", kt: "20*", desc: "Отписване амортизация при дарение"},

    # 130 - Други транзакции (Other Transactions)
    {code: "130", dt: "20*", kt: "499", desc: "Други транзакции с ДМА"},
  ]

  # Create default mappings for a company
  def self.create_default_mappings(company_id : Int64)
    DEFAULT_MAPPINGS.each do |mapping|
      SaveSaftAssetAccountMapping.create!(
        company_id: company_id,
        asset_transaction_type: mapping[:code],
        debit_account: mapping[:dt],
        credit_account: mapping[:kt],
        description: mapping[:desc],
        is_active: true
      )
    end
  end

  # Find matching mapping for a transaction
  def self.find_mapping(company_id : Int64, transaction_type : String, dt_account : String, kt_account : String) : SaftAssetAccountMapping?
    mappings = SaftAssetAccountMappingQuery.new
      .company_id(company_id)
      .asset_transaction_type(transaction_type)
      .is_active(true)

    mappings.each do |mapping|
      return mapping if mapping.matches?(dt_account, kt_account)
    end

    nil
  end

  # Detect transaction type from account correspondence
  def self.detect_transaction_type(company_id : Int64, dt_account : String, kt_account : String) : String?
    mappings = SaftAssetAccountMappingQuery.new
      .company_id(company_id)
      .is_active(true)

    mappings.each do |mapping|
      return mapping.asset_transaction_type if mapping.matches?(dt_account, kt_account)
    end

    nil
  end
end
