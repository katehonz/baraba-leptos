class Api::JournalEntries::Post < ApiAction
  post "/api/companies/:company_id/journal_entries/:entry_id/post" do
    entry = JournalEntryQuery.new.company_id(company_id).id(entry_id).first

    if entry.status == "posted"
      response.status_code = 422
      return json({success: false, error: "Journal entry already posted"})
    end

    # Validate balanced entry
    lines = JournalLineQuery.new.journal_entry_id(entry.id)
    total_debit = lines.sum(&.debit_amount)
    total_credit = lines.sum(&.credit_amount)

    if (total_debit - total_credit).abs > 0.01
      response.status_code = 422
      return json({
        success: false,
        error:   "Journal entry is not balanced. Debit: #{total_debit}, Credit: #{total_credit}",
      })
    end

    if lines.size == 0
      response.status_code = 422
      return json({success: false, error: "Journal entry has no lines"})
    end

    SaveJournalEntry.update!(entry, status: "posted")

    json({
      success: true,
      message: "Journal entry posted",
      data:    {
        id:           entry.id,
        status:       "posted",
        total_debit:  total_debit,
        total_credit: total_credit,
      },
    })
  end
end
