# Parses journal entry `lines` JSON.
# Manual entries often store only `account_code` (no `account_id`).
# Amounts may be Float64, Int64, or String depending on the writer.
module JournalLines
  def self.parse(lines_str : String?) : Array(JSON::Any)
    return [] of JSON::Any if lines_str.nil? || lines_str.empty?

    begin
      parsed = JSON.parse(lines_str)
      parsed.as_a? || [] of JSON::Any
    rescue
      [] of JSON::Any
    end
  end

  def self.number(value : JSON::Any?) : Float64
    return 0.0 if value.nil?
    value.as_f? || value.as_i64?.try(&.to_f) || value.as_s?.try(&.to_f64?) || 0.0
  end

  def self.int64(value : JSON::Any?) : Int64?
    return nil if value.nil?
    value.as_i64? || value.as_f?.try(&.to_i64) || value.as_s?.try(&.to_i64?)
  end

  def self.debit(line : JSON::Any) : Float64
    debit = number(line["debit"]?)
    debit != 0.0 ? debit : number(line["debit_amount"]?)
  end

  def self.credit(line : JSON::Any) : Float64
    credit = number(line["credit"]?)
    credit != 0.0 ? credit : number(line["credit_amount"]?)
  end

  def self.total_debit(lines_str : String?) : Float64
    parse(lines_str).sum(0.0) { |line| debit(line) }
  end

  def self.total_credit(lines_str : String?) : Float64
    parse(lines_str).sum(0.0) { |line| credit(line) }
  end

  # Amount of the journal entry from its lines (debit side = document total).
  # Falls back to the stored column only when lines have no amounts.
  def self.document_total(lines_str : String?, stored_total : Float64? = nil) : Float64
    total = total_debit(lines_str)
    return total.round(2) if total != 0.0
    (stored_total || 0.0).round(2)
  end

  # Totals from journal_lines rows if present, otherwise from the JSON `lines` column.
  def self.totals_for(entry : JournalEntry) : NamedTuple(debit: Float64, credit: Float64, count: Int32)
    table_lines = JournalLineQuery.new.journal_entry_id(entry.id)
    table_count = table_lines.select_count
    if table_count > 0
      return {
        debit:  table_lines.sum(&.debit_amount).round(2),
        credit: table_lines.sum(&.credit_amount).round(2),
        count:  table_count.to_i,
      }
    end

    parsed = parse(entry.lines)
    {
      debit:  parsed.sum { |line| debit(line) }.round(2),
      credit: parsed.sum { |line| credit(line) }.round(2),
      count:  parsed.size,
    }
  end

  def self.account_id(line : JSON::Any) : Int64?
    int64(line["account_id"]?)
  end

  def self.account_code(line : JSON::Any) : String?
    raw = line["account_code"]?
    return nil unless raw
    if s = raw.as_s?
      stripped = s.strip
      return stripped unless stripped.empty?
      return nil
    end
    if i = raw.as_i64?
      return i.to_s
    end
    nil
  end

  def self.description(line : JSON::Any) : String?
    raw = line["description"]?
    return nil unless raw
    if s = raw.as_s?
      stripped = s.strip
      return stripped unless stripped.empty?
    end
    nil
  end

  def self.counterpart_id(line : JSON::Any) : Int64?
    int64(line["counterpart_id"]?)
  end

  # Bank bookings often store only account_id (empty account_code).
  def self.resolve_account_code(line : JSON::Any, id_to_code : Hash(Int64, String)) : String
    if code = account_code(line)
      return code
    end
    if id = account_id(line)
      return id_to_code[id]? || ""
    end
    ""
  end

  def self.settlement_account?(code : String) : Bool
    return false if code.empty? || code.starts_with?("453")
    code.starts_with?("40") || code.starts_with?("41")
  end

  def self.matches_account?(line : JSON::Any, account_id : Int64, account_code : String) : Bool
    if id = self.account_id(line)
      return true if id == account_id
    end
    if code = self.account_code(line)
      return true if code == account_code
    end
    false
  end

  def self.resolve_account_id(line : JSON::Any, known_ids : Hash(Int64, T), by_code : Hash(String, Int64)) : Int64? forall T
    if id = account_id(line)
      return id if known_ids.has_key?(id)
    end
    if code = account_code(line)
      return by_code[code]?
    end
    account_id(line)
  end

  # Fill missing account_id from the company chart (by account_code).
  def self.enrich_account_ids(lines_str : String, company_id : Int64) : String
    lines = parse(lines_str)
    return lines_str if lines.empty?

    by_code = {} of String => Int64
    AccountQuery.new.company_id(company_id).each do |account|
      by_code[account.code] = account.id
    end

    changed = false
    enriched = lines.map do |line|
      hash = line.as_h? || {} of String => JSON::Any
      if account_id(line).nil?
        if code = account_code(line)
          if id = by_code[code]?
            hash = hash.dup
            hash["account_id"] = JSON::Any.new(id)
            changed = true
          end
        end
      end
      JSON::Any.new(hash)
    end

    changed ? enriched.to_json : lines_str
  end

  # Fill missing account_code from the company chart (by account_id).
  def self.enrich_account_codes(lines_str : String, company_id : Int64) : String
    lines = parse(lines_str)
    return lines_str if lines.empty?

    by_id = {} of Int64 => String
    AccountQuery.new.company_id(company_id).each do |account|
      by_id[account.id] = account.code
    end

    changed = false
    enriched = lines.map do |line|
      hash = line.as_h? || {} of String => JSON::Any
      if account_code(line).nil?
        if id = account_id(line)
          if code = by_id[id]?
            hash = hash.dup
            hash["account_code"] = JSON::Any.new(code)
            changed = true
          end
        end
      end
      JSON::Any.new(hash)
    end

    changed ? enriched.to_json : lines_str
  end

  def self.enrich(lines_str : String, company_id : Int64) : String
    enrich_account_codes(enrich_account_ids(lines_str, company_id), company_id)
  end
end
