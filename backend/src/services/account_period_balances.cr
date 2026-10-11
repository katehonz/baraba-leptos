# Unnetted НД/НК/ОД/ОК for a single period. Prior year = a second compute().
# Row is a class: Crystal Hash returns a copy of a struct, so += on a struct
# value would be discarded and every FS line would stay 0.
struct AccountPeriodBalances
  class Row
    getter account_id : Int64
    getter code : String
    property opening_debit : Float64
    property opening_credit : Float64
    property turnover_debit : Float64
    property turnover_credit : Float64

    def initialize(@account_id, @code, @opening_debit = 0.0, @opening_credit = 0.0, @turnover_debit = 0.0, @turnover_credit = 0.0)
    end

    def closing_debit : Float64
      (opening_debit + turnover_debit).round(2)
    end

    def closing_credit : Float64
      (opening_credit + turnover_credit).round(2)
    end

    def active? : Bool
      opening_debit != 0.0 || opening_credit != 0.0 || turnover_debit != 0.0 || turnover_credit != 0.0
    end
  end

  def self.ymd(t : Time) : String
    t.to_s("%Y-%m-%d")
  end

  def self.fiscal_start(company : Company, from : Time) : Time
    fy = company.fiscal_year_start_month
    fy = 1 if fy < 1 || fy > 12
    y = from.month >= fy ? from.year : from.year - 1
    Time.utc(y, fy, 1)
  end

  def self.compute(company_id : Int64, date_from : Time, date_to : Time) : Hash(String, Row)
    company = CompanyQuery.new.find(company_id)
    fiscal_start = fiscal_start(company, date_from)
    from_s = ymd(date_from)
    to_s = ymd(date_to)
    fiscal_s = ymd(fiscal_start)

    accounts_by_id = {} of Int64 => String
    code_to_id = {} of String => Int64
    AccountQuery.new.company_id(company_id).each do |a|
      accounts_by_id[a.id] = a.code
      code_to_id[a.code] = a.id
    end

    rows = {} of Int64 => Row
    accounts_by_id.each do |id, code|
      rows[id] = Row.new(id, code)
    end

    # Snapshot = max OB date ≤ fiscal_start, per account. Sum analytics on that date only.
    latest = {} of Int64 => Time
    OpeningBalanceQuery.new.company_id(company_id).each do |ob|
      next if ymd(ob.date) > fiscal_s
      if prev = latest[ob.account_id]?
        latest[ob.account_id] = ob.date if ob.date > prev
      else
        latest[ob.account_id] = ob.date
      end
    end

    OpeningBalanceQuery.new.company_id(company_id).each do |ob|
      snap = latest[ob.account_id]?
      next unless snap && ymd(ob.date) == ymd(snap)
      row = rows[ob.account_id]?
      next unless row
      row.opening_debit += ob.debit
      row.opening_credit += ob.credit
    end

    min_start_s = fiscal_s
    latest.each_value do |d|
      ds = ymd(d)
      min_start_s = ds if ds < min_start_s
    end

    JournalEntryQuery.new.company_id(company_id).status("posted").each do |entry|
      day = ymd(entry.entry_date)
      next if day > to_s
      next if day < min_start_s

      JournalLines.parse(entry.lines).each do |line|
        id = JournalLines.account_id(line)
        code = JournalLines.account_code(line)
        unless id && rows.has_key?(id)
          id = code.try { |c| code_to_id[c]? }
        end
        next unless id
        row = rows[id]?
        unless row
          resolved = code || accounts_by_id[id]?
          next unless resolved
          row = Row.new(id, resolved)
          rows[id] = row
        end

        d = JournalLines.debit(line)
        c = JournalLines.credit(line)
        next if d == 0.0 && c == 0.0

        if day < from_s
          row.opening_debit += d
          row.opening_credit += c
        else
          row.turnover_debit += d
          row.turnover_credit += c
        end
      end
    end

    by_code = {} of String => Row
    rows.each_value do |row|
      next unless row.active?
      by_code[row.code] = row
    end
    by_code
  end
end
