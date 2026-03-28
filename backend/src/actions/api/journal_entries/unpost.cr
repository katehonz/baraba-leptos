class Api::JournalEntries::Unpost < ApiAction
  post "/api/companies/:company_id/journal_entries/:entry_id/unpost" do
    entry = JournalEntryQuery.new.company_id(company_id).id(entry_id).first

    if entry.status != "posted"
      response.status_code = 422
      return json({success: false, error: "Journal entry is not posted"})
    end

    SaveJournalEntry.update!(entry, status: "draft")

    json({
      success: true,
      message: "Journal entry returned to draft",
      data:    {
        id:     entry.id,
        status: "draft",
      },
    })
  end
end
