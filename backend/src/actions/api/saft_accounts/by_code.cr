class Api::SaftAccounts::ByCode < ApiAction
  include Api::Auth::SkipRequireAuthToken

  get "/api/saft_accounts/code/:code" do
    account = SaftAccountQuery.new.by_code(code).first?

    if account
      json({
        success: true,
        data:    {
          id:           account.id,
          code:         account.code,
          name:         account.name,
          section_code: account.section_code,
          section_name: account.section_name,
          group_code:   account.group_code,
          group_name:   account.group_name,
          display_name: account.display_name,
        },
      })
    else
      response.status_code = 404
      json({success: false, error: "SAF-T account not found"})
    end
  end
end
