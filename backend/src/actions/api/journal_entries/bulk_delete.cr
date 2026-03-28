class Api::JournalEntries::BulkDelete < ApiAction
  post "/api/companies/:company_id/journal_entries/bulk_delete" do
    ids = params.from_json["ids"]?.try(&.as_a?.try(&.map(&.as_i64)))

    if ids.nil? || ids.empty?
      return json({success: false, error: "No entry IDs provided"})
    end

    deleted_count = 0
    errors = [] of String

    ids.each do |id|
      begin
        entry = JournalEntryQuery.new.company_id(company_id).id(id).first?
        next unless entry

        if entry.status == "draft"
          DeleteJournalEntry.delete!(entry)
          deleted_count += 1
        else
          errors << "Entry ##{id} is not a draft"
        end
      rescue e
        errors << "Failed to delete entry ##{id}: #{e.message}"
      end
    end

    if errors.empty?
      json({
        success:       true,
        deleted_count: deleted_count,
        message:       "Successfully deleted #{deleted_count} entries",
      })
    else
      json({
        success:       deleted_count > 0,
        deleted_count: deleted_count,
        errors:        errors,
        message:       "Deleted #{deleted_count} entries with #{errors.size} errors",
      })
    end
  end
end
