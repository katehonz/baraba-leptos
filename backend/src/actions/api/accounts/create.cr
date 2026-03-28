class Api::Accounts::Create < ApiAction
  post "/api/companies/:company_id/accounts" do
    # Parse JSON body directly
    body = request.body.try(&.gets_to_end) || "{}"
    json_data = JSON.parse(body)

    code = json_data["code"]?.try(&.as_s) || ""
    name = json_data["name"]?.try(&.as_s) || ""
    account_type = json_data["account_type"]?.try(&.as_s) || "asset"
    is_active = json_data["is_active"]?.try(&.as_bool?) || true
    saft_account_id = json_data["saft_account_id"]?.try(&.as_i64?)
    tracks_articles = json_data["tracks_articles"]?.try(&.as_bool?) || false

    account = SaveAccount.create!(
      code: code,
      name: name,
      account_type: account_type,
      is_active: is_active,
      company_id: company_id.to_i64,
      saft_account_id: saft_account_id,
      tracks_articles: tracks_articles
    )

    # Reload to get saft_account if set
    full_account = AccountQuery.new.id(account.id).preload_saft_account.first
    saft = full_account.saft_account

    json({
      success: true,
      data:    {
        id:              full_account.id,
        code:            full_account.code,
        name:            full_account.name,
        account_type:    full_account.account_type,
        is_active:       full_account.is_active,
        tracks_articles: full_account.tracks_articles,
        saft_account_id: full_account.saft_account_id,
        saft_account:    saft ? {
          id:   saft.id,
          code: saft.code,
          name: saft.name,
        } : nil,
      },
    })
  rescue ex : Avram::InvalidOperationError
    response.status_code = 422
    json({
      success: false,
      errors:  [{field: "base", messages: [ex.message]}],
    })
  end
end
