class FsEvaluator
  struct LineResult
    getter code : String
    getter parent : String?
    getter kind : String
    getter side : String
    getter name : String
    getter formula : String
    getter current : Float64
    getter prior : Float64
    getter hidden_in_export : Bool

    def initialize(@code, @parent, @kind, @side, @name, @formula, @current, @prior)
      @hidden_in_export = @current == 0.0 && @prior == 0.0 && @kind != "header"
    end
  end

  struct Result
    getter rows : Array(LineResult)
    getter warnings : Array(String)
    getter diagnostics : Hash(String, Float64)

    def initialize(@rows, @warnings, @diagnostics)
    end
  end

  def self.compute(
    company_id : Int64,
    statement : String,
    date_from : Time,
    date_to : Time,
    include_prior : Bool = true,
  ) : Result
    current_vals, warnings = evaluate_period(company_id, statement, date_from, date_to)
    prior_vals = {} of String => Float64
    if include_prior
      prior_from = shift_year(date_from, -1)
      prior_to = shift_year(date_to, -1)
      prior_vals, prior_warn = evaluate_period(company_id, statement, prior_from, prior_to)
      warnings.concat(prior_warn.map { |w| "предх. год.: #{w}" })
    end

    catalog = FsCatalog.lines(statement)
    stored = formula_map(company_id, statement)
    rows = catalog.map do |cat|
      formula = stored[cat.code]? || ""
      cur = current_vals[cat.code]? || 0.0
      pri = prior_vals[cat.code]? || 0.0
      LineResult.new(cat.code, cat.parent, cat.kind, cat.side, cat.name, formula, cur, pri)
    end

    diag = {} of String => Float64
    if statement == "balance"
      asset = current_vals["bal.t"]? || 0.0
      liab = current_vals["bal.p.t"]? || 0.0
      diag["asset_total"] = asset
      diag["liability_total"] = liab
      diag["delta"] = (asset - liab).round(2)
      diag["current_result_123"] = current_vals["bal.p.a.vi"]? || 0.0
    elsif statement == "pl"
      diag["pl_profit"] = current_vals["pl.e.profit"]? || 0.0
    end

    Result.new(rows, warnings.uniq, diag)
  end

  def self.evaluate_period(company_id : Int64, statement : String, date_from : Time, date_to : Time) : {Hash(String, Float64), Array(String)}
    catalog = FsCatalog.lines(statement)
    stored = formula_map(company_id, statement)
    balances = AccountPeriodBalances.compute(company_id, date_from, date_to)
    accounts_by_id = {} of Int64 => String
    balances.each_value { |row| accounts_by_id[row.account_id] = row.code }
    AccountQuery.new.company_id(company_id).each do |a|
      accounts_by_id[a.id] = a.code unless accounts_by_id.has_key?(a.id)
    end
    pairs = CorrespondenceFifo.pairs(company_id, date_from, date_to, accounts_by_id)

    values = {} of String => Float64
    done = {} of String => Bool
    warnings = [] of String

    catalog.each { |c| done[c.code] = false }

    max_iter = catalog.size + 4
    max_iter.times do
      progress = false
      catalog.each do |cat|
        next if done[cat.code]
        formula = stored[cat.code]? || ""
        val = if formula.blank?
                if cat.kind == "total"
                  sibling_sum(catalog, cat, values, done)
                else
                  0.0
                end
              else
                eval_formula(formula, statement, balances, pairs, values, done)
              end
        next if val.nil?
        values[cat.code] = val.not_nil!.round(2)
        done[cat.code] = true
        progress = true
      end
      break unless progress
      break if done.values.all?
    end

    catalog.each do |cat|
      unless done[cat.code]
        values[cat.code] = 0.0
        warnings << "цикъл или липсваща референция: #{cat.code}"
      end
    end

    if statement == "pl"
      if v = values["pl.e.profit"]?
        values["pl.e.profit"] = Math.max(0.0, v).round(2)
      end
      if v = values["pl.i.loss"]?
        values["pl.i.loss"] = Math.max(0.0, v).round(2)
      end
    end

    {values, warnings}
  end

  private def self.formula_map(company_id : Int64, statement : String) : Hash(String, String)
    map = {} of String => String
    FsLineFormulaQuery.new.company_id(company_id).statement(statement).each do |row|
      map[row.line_code] = row.formula
    end
    map
  end

  private def self.sibling_sum(catalog, cat, values, done) : Float64?
    parent = cat.parent
    total = 0.0
    catalog.each do |other|
      next if other.code == cat.code
      next unless other.parent == parent
      next unless other.kind == "line" || other.kind == "total"
      return nil unless done[other.code]?
      total += values[other.code]
    end
    total
  end

  private def self.eval_formula(formula, statement, balances, pairs, values, done) : Float64?
    tokens = FsFormula.parse(formula, statement)
    total = 0.0
    tokens.each do |tok|
      piece = token_value(tok, balances, pairs, values, done)
      return nil if piece.nil?
      total += piece.not_nil!
    end
    total
  rescue FsParseError
    0.0
  end

  private def self.token_value(tok : FsToken, balances, pairs, values, done) : Float64?
    raw = case tok.kind
          when "REF"
            code = tok.line_code.not_nil!
            return nil unless done[code]?
            values[code]? || 0.0
          when "CORR"
            sum_pairs(pairs, tok.account.not_nil!, tok.account2.not_nil!)
          when "D"
            sum_bal(balances, tok.account.not_nil!) { |r| r.closing_debit }
          when "C"
            sum_bal(balances, tok.account.not_nil!) { |r| r.closing_credit }
          when "TD"
            sum_bal(balances, tok.account.not_nil!) { |r| r.turnover_debit }
          when "TC"
            sum_bal(balances, tok.account.not_nil!) { |r| r.turnover_credit }
          when "OD"
            sum_bal(balances, tok.account.not_nil!) { |r| r.opening_debit }
          when "OC"
            sum_bal(balances, tok.account.not_nil!) { |r| r.opening_credit }
          else
            0.0
          end
    signed = tok.negative? ? -raw : raw
    signed.round(2)
  end

  private def self.sum_bal(balances : Hash(String, AccountPeriodBalances::Row), pattern : String, &) : Float64
    total = 0.0
    balances.each do |code, row|
      next unless AccountPattern.matches?(code, pattern)
      total += yield row
    end
    total
  end

  private def self.sum_pairs(pairs : Array(CorrespondenceFifo::Pair), debit_pat : String, credit_pat : String) : Float64
    total = 0.0
    pairs.each do |p|
      next unless AccountPattern.matches?(p.debit_code, debit_pat)
      next unless AccountPattern.matches?(p.credit_code, credit_pat)
      total += p.amount
    end
    total
  end

  private def self.shift_year(t : Time, years : Int32) : Time
    y = t.year + years
    m = t.month
    d = t.day
    max_d = Time.days_in_month(y, m)
    d = max_d if d > max_d
    Time.utc(y, m, d)
  end
end
