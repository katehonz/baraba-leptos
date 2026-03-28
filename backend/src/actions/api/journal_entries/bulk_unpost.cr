class Api::JournalEntries::BulkUnpost < ApiAction
  post "/api/companies/:company_id/journal_entries/bulk_unpost" do
    ids = params.from_json["ids"].as_a.map(&.as_i64)

    unposted_count = 0
    errors = [] of String

    ids.each do |id|
      begin
        entry = JournalEntryQuery.new.company_id(company_id).id(id).first?
        if entry
          if entry.status == "posted"
            SaveJournalEntry.update!(entry, status: "draft")
            unposted_count += 1
          else
            errors << "Entry ##{id} is not posted"
          end
        else
          errors << "Entry ##{id} not found"
        end
      rescue e
        errors << "Failed to unpost entry ##{id}: #{e.message}"
      end
    end

    if errors.empty?
      json({
        success:        true,
        unposted_count: unposted_count,
        message:        "Successfully returned #{unposted_count} entries to draft",
      })
    else
      json({
        success:        unposted_count > 0,
        unposted_count: unposted_count,
        errors:         errors,
        message:        "Returned #{unposted_count} entries to draft with #{errors.size} errors",
      })
    end
  end
end
