# Prefix match for FS formulas and (copied, not shared) SAF-T correspondences.
# 304 matches 304, 3041, 30401. 30* matches the whole 30x class.
# Does not change SaftMovementAccountMapping#matches_pattern?
module AccountPattern
  def self.matches?(account : String, pattern : String) : Bool
    return false if pattern.empty? || account.empty?
    if pattern.ends_with?("*")
      account.starts_with?(pattern.rchop("*"))
    else
      account == pattern || account.starts_with?(pattern)
    end
  end
end
