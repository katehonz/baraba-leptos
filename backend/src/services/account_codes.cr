# Chart-of-accounts order for Bulgarian НСФО/НСС numbers.
#
# Codes are compared as strings, not integers:
#   401 < 4011 < 402 < 45901 < 50301 < 7029 < 703
#
# Integer sort is wrong and yields 703, 7029, 45901, 50301
# (class 7 before class 4/5, and 703 before its neighbour 7029).
module AccountCodes
  def self.sort_key(code : String) : String
    code.strip
  end

  def self.compare(a : String, b : String) : Int32
    sort_key(a) <=> sort_key(b)
  end
end
