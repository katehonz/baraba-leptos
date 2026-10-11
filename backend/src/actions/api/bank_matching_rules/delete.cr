class Api::BankMatchingRules::Delete < ApiAction
  delete "/api/companies/:company_id/bank_matching_rules/:bank_matching_rule_id" do
    rule = BankMatchingRuleQuery.new.company_id(company_id).find(bank_matching_rule_id)
    rule.delete
    json({success: true, message: "Правилото е изтрито"})
  end
end
