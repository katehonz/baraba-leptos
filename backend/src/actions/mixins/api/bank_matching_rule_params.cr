module Api::BankMatchingRuleParams
  private def rule_payload : JSON::Any
    body = params.from_json
    body["bank_matching_rule"]? || body
  end

  private def has_key?(data : JSON::Any, key : String) : Bool
    data.as_h?.try(&.has_key?(key)) || false
  end

  private def json_str(val : JSON::Any?) : String?
    return nil if val.nil? || val.raw.nil?
    s = val.as_s?
    return nil if s.nil? || s.blank?
    s.strip
  end

  private def json_i64(val : JSON::Any?) : Int64?
    return nil if val.nil? || val.raw.nil?
    val.as_i64? || val.as_i?.try(&.to_i64) || val.as_s?.try(&.to_i64?)
  end

  private def json_i32(val : JSON::Any?) : Int32?
    json_i64(val).try(&.to_i32)
  end

  private def json_bool(val : JSON::Any?) : Bool?
    return nil if val.nil? || val.raw.nil?
    val.as_bool?
  end

  private def serialize_rule(rule : BankMatchingRule)
    account = AccountQuery.new.id(rule.account_id).first?
    counterpart = if cid = rule.counterpart_id
                    CounterpartQuery.new.id(cid).first?
                  end
    {
      id:               rule.id,
      name:             rule.name,
      keywords:         rule.keywords,
      direction:        rule.direction,
      priority:         rule.priority,
      is_active:        rule.is_active,
      account_id:       rule.account_id,
      account_code:     account.try(&.code),
      account_name:     account.try(&.name),
      counterpart_id:   rule.counterpart_id,
      counterpart_name: counterpart.try(&.name),
    }
  end
end
