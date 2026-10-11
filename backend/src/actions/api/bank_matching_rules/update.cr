class Api::BankMatchingRules::Update < ApiAction
  include Api::BankMatchingRuleParams

  put "/api/companies/:company_id/bank_matching_rules/:bank_matching_rule_id" do
    rule = BankMatchingRuleQuery.new.company_id(company_id).find(bank_matching_rule_id)
    data = rule_payload

    SaveBankMatchingRule.update(rule,
      name: json_str(data["name"]?) || rule.name,
      keywords: json_str(data["keywords"]?) || rule.keywords,
      direction: json_str(data["direction"]?) || rule.direction,
      priority: json_i32(data["priority"]?) || rule.priority,
      is_active: has_key?(data, "is_active") ? (json_bool(data["is_active"]?) != false) : rule.is_active,
      account_id: json_i64(data["account_id"]?) || rule.account_id,
      counterpart_id: has_key?(data, "counterpart_id") ? json_i64(data["counterpart_id"]?) : rule.counterpart_id,
    ) do |operation, updated|
      if operation.saved?
        json({success: true, data: serialize_rule(updated)})
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
