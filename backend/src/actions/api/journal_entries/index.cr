class Api::JournalEntries::Index < ApiAction
  get "/api/companies/:company_id/journal_entries" do
    page = (params.get?("page") || "1").to_i
    page = 1 if page < 1
    per_page = (params.get?("per_page") || "25").to_i
    per_page = [per_page, 100].min # Max 100 per page
    search = params.get?("q")

    base_query = JournalEntryQuery.new.company_id(company_id)

    # Filter by status if provided
    if status = params.get?("status")
      base_query = base_query.status(status.to_s)
    end

    # Search description, reference, document number, and counterpart
    if search && !search.empty?
      base_query = base_query.search(search)
    end

    # Filter by entry date range (inclusive from, inclusive to)
    if date_from = parse_date_param("date_from")
      base_query = base_query.entry_date.gte(date_from)
    end
    if date_to = parse_date_param("date_to")
      base_query = base_query.entry_date.lt(date_to + 1.day)
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

  private def parse_date_param(name : String) : Time?
    raw = params.get?(name).try(&.to_s).try(&.strip)
    return nil if raw.nil? || raw.empty?
    Time.parse(raw, "%Y-%m-%d", Time::Location::UTC)
  rescue
    nil
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
      total_amount:     JournalLines.document_total(e.lines, e.total_amount),
      counterpart_name: counterpart_name,
    }
  end
end
