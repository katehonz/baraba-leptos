class Api::BankMatchingRules::CreateDefaults < ApiAction
  post "/api/companies/:company_id/bank_matching_rules/create_defaults" do
    created = BankMatchingService.create_defaults(company_id.to_i64)
    json({
      success: true,
      data:    {created: created},
      message: created > 0 ? "Добавени са #{created} стандартни правила" : "Стандартните правила вече съществуват (или липсват сметките в сметкоплана)",
    })
  end
end
