class Api::Accounts::Index < ApiAction
  get "/api/companies/:company_id/accounts" do
    page = (params.get?("page") || "1").to_i
    per_page = (params.get?("per_page") || "50").to_i
    per_page = [per_page, 1000].min # Max 1000 per page
    search = params.get?("q")
    type_filter = params.get?("type")

    base_query = AccountQuery.new.company_id(company_id)

    # Apply search filter
    if search && !search.empty?
      search_term = "%#{search}%"
      base_query = base_query.where("code ILIKE ? OR name ILIKE ?", search_term, search_term)
    end

    # Apply type filter
    if type_filter && !type_filter.empty?
      base_query = base_query.account_type(type_filter)
    end

    total = base_query.select_count
    total_pages = (total.to_f / per_page).ceil.to_i
    offset = (page - 1) * per_page

    accounts = base_query
      .preload_saft_account
      .limit(per_page)
      .offset(offset)

    json({
      success:     true,
      data:        accounts.map { |a| account_json(a) },
      pagination:  {
        page:        page,
        per_page:    per_page,
        total:       total,
        total_pages: total_pages,
      },
    })
  end

  private def account_json(account : Account)
    saft = account.saft_account
    {
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
    }
  end
end
