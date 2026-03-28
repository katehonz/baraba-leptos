# SAF-T Movement Account Mapping
# Дефинира кои Дт/Кт кореспонденции се отнасят към кой тип движение
# Позволява множество кореспонденции за един тип движение
class SaftMovementAccountMapping < BaseModel
  table do
    belongs_to company : Company

    # SAF-T Movement Type Code (10=Покупка, 30=Продажба, etc.)
    column movement_type_code : String

    # Account correspondence pattern
    column debit_account : String      # e.g., "302", "302*", "30*"
    column credit_account : String     # e.g., "401", "401*", "40*"

    # Optional: specific analytical accounts
    column debit_analytical : String?  # e.g., "3021", "30211"
    column credit_analytical : String? # e.g., "4011"

    # Description for this mapping
    column description : String?

    # Active flag
    column is_active : Bool = true
  end

  # Check if a journal entry line matches this mapping
  def matches?(dt_account : String, kt_account : String) : Bool
    matches_pattern?(dt_account, debit_account, debit_analytical) &&
      matches_pattern?(kt_account, credit_account, credit_analytical)
  end

  private def matches_pattern?(account : String, pattern : String, analytical : String?) : Bool
    # If analytical is specified, check exact match
    if analytical && !analytical.empty?
      return account == analytical || account.starts_with?(analytical)
    end

    # Check pattern (supports wildcard *)
    if pattern.ends_with?("*")
      prefix = pattern.rchop("*")
      account.starts_with?(prefix)
    else
      account == pattern || account.starts_with?(pattern)
    end
  end
end
