class SaveJournalLine < JournalLine::SaveOperation
  permit_columns debit_amount, credit_amount, description, journal_entry_id, account_id

  before_save do
    validate_required account_id

    # Either debit or credit must be > 0, but not both
    debit = debit_amount.value || 0.0
    credit = credit_amount.value || 0.0

    if debit > 0 && credit > 0
      debit_amount.add_error("cannot have both debit and credit")
    end

    if debit == 0 && credit == 0
      debit_amount.add_error("must have either debit or credit amount")
    end
  end
end
