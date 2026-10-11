# Service to map account correspondences to SAF-T movement types
class SaftMovementMapper
  # Default mappings for Bulgarian accounting (can be customized per company)
  DEFAULT_MAPPINGS = [
    # 10 = Покупка (Purchase)
    {code: "10", dt: "302", kt: "401", desc: "Материали от доставчици"},
    {code: "10", dt: "303", kt: "401", desc: "Продукция от доставчици"},
    {code: "10", dt: "304", kt: "401", desc: "Стоки от доставчици"},
    {code: "10", dt: "301", kt: "401", desc: "Материални запаси от доставчици"},

    # 20 = Материални запаси от производство (Inventory from Production)
    {code: "20", dt: "303", kt: "611", desc: "Продукция от производство"},
    {code: "20", dt: "302", kt: "611", desc: "Материали от производство"},

    # 30 = Продажба (Sale)
    {code: "30", dt: "701", kt: "304", desc: "Продажба на стоки"},
    {code: "30", dt: "701", kt: "303", desc: "Продажба на продукция"},
    {code: "30", dt: "702", kt: "304", desc: "Приходи от продажби на стоки"},
    {code: "30", dt: "411", kt: "701", desc: "Вземания от клиенти"},

    # 40 = Връщане на продадени продукти (Return of Sold Products)
    {code: "40", dt: "304", kt: "701", desc: "Върнати стоки"},
    {code: "40", dt: "303", kt: "701", desc: "Върната продукция"},

    # 50 = Връщане на закупени продукти (Return of Purchased Products)
    {code: "50", dt: "401", kt: "302", desc: "Върнати материали към доставчик"},
    {code: "50", dt: "401", kt: "304", desc: "Върнати стоки към доставчик"},

    # 70 = Материални запаси за производство (Inventory for Production)
    {code: "70", dt: "601", kt: "302", desc: "Разход на материали за производство"},
    {code: "70", dt: "611", kt: "302", desc: "Вложени материали в производство"},

    # 80 = Вътрешен трансфер (Internal Transfer)
    {code: "80", dt: "302", kt: "302", desc: "Вътрешен трансфер на материали"},
    {code: "80", dt: "304", kt: "304", desc: "Вътрешен трансфер на стоки"},

    # 110 = Положителна корекция от инвентаризация (Positive Inventory Adjustment)
    {code: "110", dt: "302", kt: "709", desc: "Излишък на материали"},
    {code: "110", dt: "304", kt: "709", desc: "Излишък на стоки"},

    # 120 = Отрицателна корекция от инвентаризация (Negative Inventory Adjustment)
    {code: "120", dt: "609", kt: "302", desc: "Липса на материали"},
    {code: "120", dt: "609", kt: "304", desc: "Липса на стоки"},

    # 160 = Брак (Scrap)
    {code: "160", dt: "609", kt: "302", desc: "Бракувани материали"},
    {code: "160", dt: "609", kt: "304", desc: "Бракувани стоки"},
  ]

  def initialize(@company_id : Int64)
  end

  # Find movement type code for given debit/credit accounts
  def find_movement_type(debit_account : String, credit_account : String) : String?
    # First check company-specific mappings
    mapping = SaftMovementAccountMappingQuery.new
      .company_id(@company_id)
      .is_active(true)
      .to_a
      .find { |m| m.matches?(debit_account, credit_account) }

    mapping.try(&.movement_type_code)
  end

  # Get all mappings for a company
  def get_mappings : Array(SaftMovementAccountMapping)
    SaftMovementAccountMappingQuery.new
      .company_id(@company_id)
      .is_active(true)
      .movement_type_code.asc_order
      .to_a
  end

  # Create default mappings for a company
  def self.create_default_mappings(company_id : Int64)
    DEFAULT_MAPPINGS.each do |mapping|
      SaveSaftMovementAccountMapping.create!(
        company_id: company_id,
        movement_type_code: mapping[:code],
        debit_account: mapping[:dt],
        credit_account: mapping[:kt],
        description: mapping[:desc],
        is_active: true
      )
    end
  end

  # Extract correspondences from journal entries and categorize by movement type
  def extract_correspondences_for_period(year : Int32, month : Int32) : Hash(String, Array(CorrespondenceEntry))
    result = Hash(String, Array(CorrespondenceEntry)).new { |h, k| h[k] = [] of CorrespondenceEntry }

    # Get all journal entries for the period
    period_start = Time.local(year, month, 1)
    period_end = period_start.at_end_of_month

    entries = JournalEntryQuery.new
      .company_id(@company_id)
      .entry_date.gte(period_start)
      .entry_date.lte(period_end)
      .to_a

    entries.each do |entry|
      next unless entry.lines

      begin
        lines = JSON.parse(entry.lines.not_nil!)
        next unless lines.as_a?

        # Extract debit and credit lines
        debit_lines = [] of {String, Float64}
        credit_lines = [] of {String, Float64}

        lines.as_a.each do |line|
          account = line["account_code"]?.try(&.as_s?) || ""
          debit = line["debit"]?.try(&.as_f?) || 0.0
          credit = line["credit"]?.try(&.as_f?) || 0.0

          debit_lines << {account, debit} if debit > 0
          credit_lines << {account, credit} if credit > 0
        end

        # Create correspondences (each debit with each credit)
        debit_lines.each do |dt_account, dt_amount|
          credit_lines.each do |kt_account, kt_amount|
            # Use the smaller amount as the correspondence amount
            amount = Math.min(dt_amount, kt_amount)
            next if amount <= 0

            movement_type = find_movement_type(dt_account, kt_account) || "180" # Default to "Other"

            result[movement_type] << CorrespondenceEntry.new(
              debit_account: dt_account,
              credit_account: kt_account,
              amount: amount,
              date: entry.entry_date,
              description: entry.description || ""
            )
          end
        end
      rescue
        # Skip entries with invalid JSON
      end
    end

    result
  end

  record CorrespondenceEntry,
    debit_account : String,
    credit_account : String,
    amount : Float64,
    date : Time,
    description : String
end
