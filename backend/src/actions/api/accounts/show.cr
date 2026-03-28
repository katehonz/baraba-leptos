class Api::Accounts::Show < ApiAction
  get "/api/companies/:company_id/accounts/:account_id" do
    account = AccountQuery.new.company_id(company_id).id(account_id).preload_saft_account.first?

    if account
      saft = account.saft_account
      json({
        success: true,
        data:    {
          id:              account.id,
          code:            account.code,
          name:            account.name,
          account_type:    account.account_type,
          is_active:       account.is_active,
          tracks_articles: account.tracks_articles,
          saft_account_id: account.saft_account_id,
          saft_account:    saft ? {
            id:   saft.id,
            code: saft.code,
            name: saft.name,
          } : nil,
        },
      })
    else
      response.status_code = 404
      json({success: false, error: "Account not found"})
    end
  end
end
