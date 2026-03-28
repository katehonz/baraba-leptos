# Bulk delete VAT journal entries
class Api::VatJournalEntries::BulkDelete < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/companies/:company_id/vat_returns/:vat_return_id/journal_entries/bulk_delete" do
    cid = company_id.to_i64
    vrid = vat_return_id.to_i64

    # Verify VAT return exists
    vat_return = VatReturnQuery.new
      .company_id(cid)
      .id(vrid)
      .first?

    unless vat_return
      return json({success: false, error: "VAT return not found"}, status: 404)
    end

    # Parse request body for entry IDs
    body_str = request.body.try(&.gets_to_end) || "{}"
    json_body = JSON.parse(body_str)
    ids = json_body["ids"]?.try(&.as_a?) || [] of JSON::Any

    if ids.empty?
      return json({success: false, error: "No entry IDs provided"})
    end

    deleted = 0
    entry_types = Set(String).new

    ids.each do |id_val|
      entry_id = id_val.as_i64? || next

      entry = VatJournalEntryQuery.new
        .company_id(cid)
        .vat_return_id(vrid)
        .id(entry_id)
        .first?

      if entry
        entry_types << entry.entry_type
        entry.delete
        deleted += 1
      end
    end

    # Renumber remaining entries for affected types
    entry_types.each do |etype|
      row = 1
      VatJournalEntryQuery.new
        .vat_return_id(vrid)
        .entry_type(etype)
        .row_number.asc_order
        .each do |e|
          if e.row_number != row
            SaveVatJournalEntry.update!(e, row_number: row)
          end
          row += 1
        end
    end

    json({
      success: true,
      deleted: deleted,
      message: "Изтрити #{deleted} записа",
    })
  rescue ex
    json({success: false, error: ex.message})
  end
end
