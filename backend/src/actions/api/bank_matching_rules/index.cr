class Api::BankMatchingRules::Index < ApiAction
  include Api::BankMatchingRuleParams

  get "/api/companies/:company_id/bank_matching_rules" do
    rules = BankMatchingRuleQuery.new
      .company_id(company_id)
      .priority.desc_order
      .id.asc_order

    json({
      success: true,
      data:    rules.map { |r| serialize_rule(r) },
    })
  end
end
