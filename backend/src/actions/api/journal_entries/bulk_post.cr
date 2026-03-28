class Api::JournalEntries::BulkPost < ApiAction
  post "/api/companies/:company_id/journal_entries/bulk_post" do
    ids = params.from_json["ids"]?.try(&.as_a?.try(&.map(&.as_i64)))

    if ids.nil? || ids.empty?
      return json({success: false, error: "No entry IDs provided"})
    end

    posted_count = 0
    errors = [] of String

    ids.each do |id|
      begin
        entry = JournalEntryQuery.new.company_id(company_id).id(id).first?
        next unless entry

        if entry.status == "draft"
          SaveJournalEntry.update!(entry, status: "posted")
          posted_count += 1
        else
          errors << "Entry ##{id} is not a draft"
        end
      rescue e
        errors << "Failed to post entry ##{id}: #{e.message}"
      end
    end

    if errors.empty?
      json({
        success:      true,
        posted_count: posted_count,
        message:      "Successfully posted #{posted_count} entries",
      })
    else
      json({
        success:      posted_count > 0,
        posted_count: posted_count,
        errors:       errors,
        message:      "Posted #{posted_count} entries with #{errors.size} errors",
      })
    end
  end
end
