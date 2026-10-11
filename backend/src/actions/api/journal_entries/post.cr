class Api::JournalEntries::Post < ApiAction
  post "/api/companies/:company_id/journal_entries/:entry_id/post" do
    entry = JournalEntryQuery.new.company_id(company_id).id(entry_id).first

    if entry.status == "posted"
      response.status_code = 422
      return json({success: false, error: "Записът вече е осчетоводен"})
    end

    totals = JournalLines.totals_for(entry)

    if totals[:count] == 0
      response.status_code = 422
      return json({success: false, error: "Записът няма редове"})
    end

    if (totals[:debit] - totals[:credit]).abs > 0.01
      response.status_code = 422
      return json({
        success: false,
        error:   "Записът не е балансиран. Дебит: #{totals[:debit]}, Кредит: #{totals[:credit]}",
      })
    end

    SaveJournalEntry.update!(entry, status: "posted")

    json({
      success: true,
      message: "Записът е осчетоводен",
      data:    {
        id:           entry.id,
        status:       "posted",
        total_debit:  totals[:debit],
        total_credit: totals[:credit],
      },
    })
  end
end
