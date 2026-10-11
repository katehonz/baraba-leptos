class Api::JournalEntries::AddLine < ApiAction
  post "/api/companies/:company_id/journal_entries/:entry_id/lines" do
    entry = JournalEntryQuery.new.company_id(company_id).id(entry_id).first

    if entry.status == "posted"
      response.status_code = 422
      return json({success: false, error: "Cannot modify posted journal entry"})
    end

    SaveJournalLine.create(params, journal_entry_id: entry.id) do |operation, line|
      if line
        json({
          success: true,
          data:    {
            id:            line.id,
            account_id:    line.account_id,
            debit_amount:  line.debit_amount,
            credit_amount: line.credit_amount,
            description:   line.description,
          },
        })
      else
        response.status_code = 422
        json({
          success: false,
          errors:  operation.errors.map { |key, errors| {field: key, messages: errors} },
        })
      end
    end
  end
end
