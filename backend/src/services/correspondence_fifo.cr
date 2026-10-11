module CorrespondenceFifo
  record Pair, debit_code : String, credit_code : String, amount : Float64

  class Item
    property code : String
    property rem : Float64

    def initialize(@code, @rem)
    end
  end

  def self.pairs(
    company_id : Int64,
    date_from : Time,
    date_to : Time,
    accounts_by_id : Hash(Int64, String),
  ) : Array(Pair)
    out = [] of Pair

    from_s = AccountPeriodBalances.ymd(date_from)
    to_s = AccountPeriodBalances.ymd(date_to)
    JournalEntryQuery.new.company_id(company_id).status("posted").each do |entry|
      day = AccountPeriodBalances.ymd(entry.entry_date)
      next if day < from_s || day > to_s
      out.concat(pairs_for_entry(entry, accounts_by_id))
    end
    out
  end

  def self.pairs_for_entry(entry : JournalEntry, accounts_by_id : Hash(Int64, String)) : Array(Pair)
    debits = [] of Item
    credits = [] of Item

    JournalLines.parse(entry.lines).each do |line|
      code = JournalLines.account_code(line)
      if code.nil? || code.blank?
        if id = JournalLines.account_id(line)
          code = accounts_by_id[id]?
        end
      end
      next if code.nil? || code.blank?

      d = JournalLines.debit(line)
      c = JournalLines.credit(line)
      debits << Item.new(code.not_nil!, d) if d > 0
      credits << Item.new(code.not_nil!, c) if c > 0
    end

    result = [] of Pair
    di = 0
    ci = 0
    while di < debits.size && ci < credits.size
      if debits[di].rem <= 0
        di += 1
        next
      end
      if credits[ci].rem <= 0
        ci += 1
        next
      end
      amt = Math.min(debits[di].rem, credits[ci].rem).round(2)
      result << Pair.new(debits[di].code, credits[ci].code, amt)
      debits[di].rem = (debits[di].rem - amt).round(2)
      credits[ci].rem = (credits[ci].rem - amt).round(2)
    end
    result
  end
end
