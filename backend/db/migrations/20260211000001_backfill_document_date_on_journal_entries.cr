class BackfillDocumentDateOnJournalEntries::V20260211000001 < Avram::Migrator::Migration::V1
  def migrate
    execute "UPDATE journal_entries SET document_date = entry_date WHERE document_date IS NULL"
  end

  def rollback
    execute "UPDATE journal_entries SET document_date = NULL WHERE document_date = entry_date"
  end
end
