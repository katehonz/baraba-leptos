class Api::JournalEntries::Index < ApiAction
  get "/api/companies/:company_id/journal_entries" do
    page = (params.get?("page") || "1").to_i
    per_page = (params.get?("per_page") || "25").to_i
    per_page = [per_page, 100].min # Max 100 per page
    search = params.get?("q")

    base_query = JournalEntryQuery.new.company_id(company_id)

    # Filter by status if provided
    if status = params.get?("status")
      base_query = base_query.status(status.to_s)
    end

    # Apply search filter
    if search && !search.empty?
      search_term = "%#{search}%"
      base_query = base_query.where("description ILIKE ? OR reference ILIKE ?", search_term, search_term)
    end

    total = base_query.select_count
    total_pages = (total.to_f / per_page).ceil.to_i
    offset = (page - 1) * per_page

    entries = base_query
      .entry_date.desc_order
      .limit(per_page)
      .offset(offset)

    json({
      success:    true,
      data:       entries.map { |e| entry_json(e) },
      pagination: {
        page:        page,
        per_page:    per_page,
        total:       total,
        total_pages: total_pages,
      },
    })
  end

  private def entry_json(e : JournalEntry)
    counterpart_name = nil
    if cid = e.counterpart_id
      cp = CounterpartQuery.new.id(cid).first?
      counterpart_name = cp.try(&.name)
    end

    {
      id:               e.id,
      entry_date:       e.entry_date,
      description:      e.description,
      reference:        e.reference,
      status:           e.status,
      document_number:  e.document_number,
      total_amount:     e.total_amount,
      counterpart_name: counterpart_name,
    }
  end
end
