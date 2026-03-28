class CreateJournalLines::V20250125000007 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(JournalLine) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to journal_entry : JournalEntry, on_delete: :cascade
      add_belongs_to account : Account, on_delete: :restrict
      add debit_amount : Float64, default: 0.0
      add credit_amount : Float64, default: 0.0
      add description : String?
    end

    execute "CREATE INDEX IF NOT EXISTS journal_lines_journal_entry_id_index ON journal_lines (journal_entry_id)"
  end

  def rollback
    drop table_for(JournalLine)
  end
end
