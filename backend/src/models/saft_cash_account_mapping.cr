# SAF-T Cash Account Mapping
# Кореспонденции Дт/Кт за движения на парични средства (каси, банки)
class SaftCashAccountMapping < BaseModel
  table do
    belongs_to company : Company
    column cash_movement_type : String    # SAF-T Cash Movement Type Code (10=Каса приход, 42=Банка приход, etc.)
    column debit_account : String         # e.g., "501", "503*", "50*"
    column credit_account : String        # e.g., "411", "401*", "40*"
    column debit_analytical : String?     # Optional analytical account pattern
    column credit_analytical : String?    # Optional analytical account pattern
    column description : String?          # Human-readable description
    column is_active : Bool = true
  end

  # Check if this mapping matches given account codes
  def matches?(dt_account : String, kt_account : String) : Bool
    matches_pattern?(dt_account, debit_account, debit_analytical) &&
      matches_pattern?(kt_account, credit_account, credit_analytical)
  end

  private def matches_pattern?(account : String, pattern : String, analytical : String?) : Bool
    # Handle wildcard patterns
    if pattern.ends_with?("*")
      prefix = pattern.rstrip('*')
      return false unless account.starts_with?(prefix)
    else
      return false unless account == pattern || account.starts_with?(pattern)
    end

    # Check analytical if specified
    if analytical && !analytical.empty?
      if analytical.ends_with?("*")
        prefix = analytical.rstrip('*')
        return account.starts_with?(prefix)
      else
        return account == analytical || account.starts_with?(analytical)
      end
    end

    true
  end
end
