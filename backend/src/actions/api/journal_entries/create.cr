class Api::JournalEntries::Create < ApiAction
  post "/api/companies/:company_id/journal_entries" do
    SaveJournalEntry.create(params, company_id: company_id.to_i64) do |operation, entry|
      if entry
        # Use JSONB serializer
        json({
          success: true,
          data: JsonbSerializer.serialize_journal_entry(entry),
        })
      else
        response.status_code = 422
        json({
          success: false,
          errors: operation.errors.map { |key, errors| {field: key, messages: errors} },
        })
      end
    end
  end
end
