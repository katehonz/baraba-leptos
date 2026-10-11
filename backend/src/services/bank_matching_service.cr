class BankMatchingService
  DEFAULT_TEMPLATES = [
    {
      name:     "POS покупки",
      keywords: "POS,ПОС,pos трансакция,покупка на пос,pos purchase,card purchase,покупка с карта,покупка с дк",
      direction: "debit",
      codes:    ["447", "446", "50902"],
      priority: 100,
    },
    {
      name:     "ATM теглене",
      keywords: "ATM,АТМ,atm трансакция,теглене,withdrawal,cash withdrawal,теглене на каса,теглене на пари",
      direction: "debit",
      codes:    ["501"],
      priority: 90,
    },
    {
      name:     "Банкови такси",
      keywords: "автоматична такса,такса теглене,такса за поддръжка,комисионна,bank fee,commission,месечна такса,мес. такса,monthly fee",
      direction: "debit",
      codes:    ["629"],
      priority: 80,
    },
    {
      name:     "Получени преводи от клиенти",
      keywords: "получен директен превод,получен превод,sct instant получен,incoming credit,credit transfer received",
      direction: "credit",
      codes:    ["411"],
      priority: 70,
    },
    {
      name:     "Издадени преводи към доставчици",
      keywords: "издаден превод,sct instant издаден,изходящ превод,outgoing credit,issued transfer",
      direction: "debit",
      codes:    ["401"],
      priority: 60,
    },
    {
      name:     "Аванси",
      keywords: "аванс,avans,advance",
      direction: "debit",
      codes:    ["402"],
      priority: 50,
    },
  ]

  def self.matches_keywords?(text : String, keywords : String) : Bool
    keywords.split(',').any? do |raw|
      kw = raw.strip
      !kw.empty? && BankTextNormalizer.contains_keyword?(text, kw)
    end
  end

  def self.direction_matches?(direction : String, amount : Float64) : Bool
    case direction
    when "debit"
      amount < 0
    when "credit"
      amount > 0
    else
      true
    end
  end

  def self.search_text(tx : BankTransaction) : String
    [tx.description, tx.contra_name, tx.reference].compact.join(" ")
  end

  def self.find_rule(company_id : Int64, tx : BankTransaction) : BankMatchingRule?
    text = search_text(tx)
    return nil if text.blank?

    rules = BankMatchingRuleQuery.new
      .company_id(company_id)
      .is_active(true)
      .priority.desc_order

    rules.each do |rule|
      next unless direction_matches?(rule.direction, tx.amount)
      return rule if matches_keywords?(text, rule.keywords)
    end
    nil
  end

  def self.find_counterpart(company_id : Int64, contra_name : String?) : Counterpart?
    return nil if contra_name.nil? || contra_name.blank?
    key = BankTextNormalizer.company_key(contra_name.not_nil!)
    return nil if key.size < 3

    CounterpartQuery.new.company_id(company_id).is_active(true).each do |cp|
      cp_key = BankTextNormalizer.company_key(cp.name)
      next if cp_key.empty?
      return cp if cp_key == key
      if cp_key.size >= 5 && key.size >= 5
        return cp if key.includes?(cp_key) || cp_key.includes?(key)
      end
    end
    nil
  end

  def self.find_account_by_codes(company_id : Int64, codes : Array(String)) : Account?
    codes.each do |code|
      if acc = AccountQuery.new.company_id(company_id).code(code).first?
        return acc
      end
    end
    nil
  end

  def self.create_defaults(company_id : Int64) : Int32
    existing_names = Set(String).new
    BankMatchingRuleQuery.new.company_id(company_id).each do |r|
      existing_names << r.name
    end

    created = 0
    DEFAULT_TEMPLATES.each do |tpl|
      next if existing_names.includes?(tpl[:name])
      account = find_account_by_codes(company_id, tpl[:codes])
      next unless account

      SaveBankMatchingRule.create!(
        company_id: company_id,
        name: tpl[:name],
        keywords: tpl[:keywords],
        direction: tpl[:direction],
        account_id: account.id,
        priority: tpl[:priority],
        is_active: true
      )
      created += 1
    end
    created
  end

  def self.needs_processing?(tx : BankTransaction, bank_account : BankAccount) : Bool
    return true if tx.journal_entry_id.nil?

    buffer_id = bank_account.buffer_account_id
    return false unless buffer_id

    entry = JournalEntryQuery.new.find(tx.journal_entry_id.not_nil!)
    lines_json = entry.lines
    return false if lines_json.empty? || lines_json == "[]"

    JSON.parse(lines_json).as_a.any? do |line|
      line["account_id"]?.try(&.as_i64?) == buffer_id
    end
  end

  def self.auto_allocate(company_id : Int64) : NamedTuple(
    matched: Int32,
    unmatched: Int32,
    errors: Int32,
    counterpart_hits: Int32,
    details: Array(Hash(String, String)),
  )
    bank_accounts_map = Hash(Int64, BankAccount).new
    BankAccountQuery.new.company_id(company_id).each do |ba|
      bank_accounts_map[ba.id] = ba
    end

    matched = 0
    unmatched = 0
    errors = 0
    counterpart_hits = 0
    details = [] of Hash(String, String)

    return {
      matched: matched,
      unmatched: unmatched,
      errors: errors,
      counterpart_hits: counterpart_hits,
      details: details,
    } if bank_accounts_map.empty?

    BankTransactionQuery.new.bank_account_id.in(bank_accounts_map.keys).each do |tx|
      bank_account = bank_accounts_map[tx.bank_account_id]
      next unless needs_processing?(tx, bank_account)

      rule = find_rule(company_id, tx)
      unless rule
        unmatched += 1
        if details.size < 50
          snippet = search_text(tx)
          snippet = snippet.size > 80 ? snippet[0, 80] : snippet
          details << {"id" => tx.id.to_s, "status" => "unmatched", "text" => snippet}
        end
        next
      end

      counterpart = if cid = rule.counterpart_id
                      CounterpartQuery.new.id(cid).first?
                    else
                      find_counterpart(company_id, tx.contra_name)
                    end
      counterpart_hits += 1 if counterpart

      begin
        if tx.journal_entry_id
          BankTransactionAllocator.reallocate!(company_id, tx, rule.account_id, counterpart.try(&.id))
        else
          BankTransactionAllocator.book!(company_id, tx, rule.account_id, counterpart.try(&.id))
        end
        matched += 1
        if details.size < 50
          details << {
            "id"     => tx.id.to_s,
            "status" => "matched",
            "rule"   => rule.name,
          }
        end
      rescue ex
        errors += 1
        if details.size < 50
          details << {
            "id"     => tx.id.to_s,
            "status" => "error",
            "error"  => ex.message || "грешка",
          }
        end
      end
    end

    {
      matched: matched,
      unmatched: unmatched,
      errors: errors,
      counterpart_hits: counterpart_hits,
      details: details,
    }
  end
end
