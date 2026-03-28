# Delete a VAT journal entry
class Api::VatJournalEntries::Delete < ApiAction
  include Api::Auth::SkipRequireAuthToken

  delete "/api/companies/:company_id/vat_returns/:vat_return_id/journal_entries/:journal_entry_id" do
    cid = company_id.to_i64
    vrid = vat_return_id.to_i64
    eid = journal_entry_id.to_i64

    # Find the entry
    entry = VatJournalEntryQuery.new
      .company_id(cid)
      .vat_return_id(vrid)
      .id(eid)
      .first?

    unless entry
      return json({success: false, error: "Journal entry not found"}, status: 404)
    end

    # Delete the entry
    entry.delete

    # Renumber remaining entries
    remaining_entries = VatJournalEntryQuery.new
      .vat_return_id(vrid)
      .entry_type(entry.entry_type)
      .row_number.asc_order

    row = 1
    remaining_entries.each do |e|
      if e.row_number != row
        SaveVatJournalEntry.update!(e, row_number: row)
      end
      row += 1
    end

    json({
      success: true,
      message: "Journal entry deleted",
    })
  rescue ex
    json({success: false, error: ex.message})
  end
end
