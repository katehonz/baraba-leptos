class BankTransactionDeleter
  class Error < Exception
  end

  def self.delete!(transaction : BankTransaction) : Nil
    je_id = transaction.journal_entry_id
    SaveBankTransaction.update!(transaction, journal_entry_id: nil)

    if je_id
      if entry = JournalEntryQuery.new.id(je_id).first?
        if entry.status == "posted"
          SaveJournalEntry.update!(entry, status: "draft")
        end
        begin
          entry.delete
        rescue
        end
      end
    end

    BankTransactionQuery.new.find(transaction.id).delete
  end
end
