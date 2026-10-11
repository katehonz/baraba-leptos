# SAF-T Asset Account Mapping
# Кореспонденции Дт/Кт за движения на дълготрайни активи
class SaftAssetAccountMapping < BaseModel
  table do
    belongs_to company : Company
    column asset_transaction_type : String  # SAF-T Asset Transaction Type Code (10=Придобиване, 20=Продажба, etc.)
    column debit_account : String           # e.g., "203", "203*", "20*"
    column credit_account : String          # e.g., "401", "401*", "40*"
    column debit_analytical : String?       # Optional analytical account pattern
    column credit_analytical : String?      # Optional analytical account pattern
    column description : String?            # Human-readable description
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
      return false unless account == pattern
    end

    # Check analytical if specified
    if analytical && !analytical.empty?
      if analytical.ends_with?("*")
        prefix = analytical.rstrip('*')
        # Would need analytical account from transaction to check
        # For now, just pattern validation
      end
    end

    true
  end
end
