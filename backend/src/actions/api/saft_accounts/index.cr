class Api::SaftAccounts::Index < ApiAction
  include Api::Auth::SkipRequireAuthToken

  get "/api/saft_accounts" do
    accounts = SaftAccountQuery.new.ordered_by_code
    json({
      success: true,
      data:    accounts.map { |a| saft_account_json(a) },
      count:   accounts.size,
    })
  end

  private def saft_account_json(account : SaftAccount)
    {
      id:           account.id,
      code:         account.code,
      name:         account.name,
      section_code: account.section_code,
      section_name: account.section_name,
      group_code:   account.group_code,
      group_name:   account.group_name,
      display_name: account.display_name,
    }
  end
end
