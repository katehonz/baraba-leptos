class Api::BankMatchingRules::Create < ApiAction
  include Api::BankMatchingRuleParams

  post "/api/companies/:company_id/bank_matching_rules" do
    data = rule_payload
    account_id = json_i64(data["account_id"]?)
    unless account_id
      response.status_code = 422
      return json({success: false, errors: [{field: "account_id", messages: ["is required"]}]})
    end

    SaveBankMatchingRule.create(
      company_id: company_id.to_i64,
      name: json_str(data["name"]?) || "",
      keywords: json_str(data["keywords"]?) || "",
      direction: json_str(data["direction"]?) || "any",
      priority: json_i32(data["priority"]?) || 0,
      is_active: json_bool(data["is_active"]?) != false,
      account_id: account_id.not_nil!,
      counterpart_id: json_i64(data["counterpart_id"]?),
    ) do |operation, rule|
      if rule
        json({success: true, data: serialize_rule(rule)})
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
