class Api::Accounts::Update < ApiAction
  put "/api/companies/:company_id/accounts/:account_id" do
    account = AccountQuery.new.id(account_id).company_id(company_id).first?

    unless account
      response.status_code = 404
      return json({success: false, error: "Сметката не е намерена"})
    end

    # Parse JSON body directly
    body = request.body.try(&.gets_to_end) || "{}"
    json_data = JSON.parse(body)

    saft_id = if json_data.as_h.has_key?("saft_account_id")
                json_data["saft_account_id"]?.try(&.as_i64?)
              else
                account.saft_account_id
              end

    updated_account = SaveAccount.update!(account,
      code: json_data["code"]?.try(&.as_s) || account.code,
      name: json_data["name"]?.try(&.as_s) || account.name,
      account_type: json_data["account_type"]?.try(&.as_s) || account.account_type,
      is_active: json_data["is_active"]?.try(&.as_bool?) || account.is_active,
      saft_account_id: saft_id,
      tracks_articles: json_data["tracks_articles"]?.try(&.as_bool?) || account.tracks_articles
    )

    # Reload with preloaded saft_account
    reloaded = AccountQuery.new.id(updated_account.id).preload_saft_account.first
    saft = reloaded.saft_account

    json({
      success: true,
      data:    {
        id:              reloaded.id,
        code:            reloaded.code,
        name:            reloaded.name,
        account_type:    reloaded.account_type,
        is_active:       reloaded.is_active,
        tracks_articles: reloaded.tracks_articles,
        saft_account_id: reloaded.saft_account_id,
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
