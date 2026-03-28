class Api::JournalEntries::Delete < ApiAction
  delete "/api/companies/:company_id/journal_entries/:entry_id" do
    entry = JournalEntryQuery.new.company_id(company_id).id(entry_id).first

    if entry.status == "posted"
      response.status_code = 422
      return json({success: false, error: "Cannot delete posted journal entry"})
    end

    DeleteJournalEntry.delete!(entry)

    json({success: true, message: "Journal entry deleted"})
  end
end
