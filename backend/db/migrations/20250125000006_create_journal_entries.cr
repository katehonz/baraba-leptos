class CreateJournalEntries::V20250125000006 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(JournalEntry) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add entry_date : Time
      add description : String
      add reference : String?
      add status : String, default: "draft"
    end

    create_index :journal_entries, [:company_id, :entry_date]
  end

  def rollback
    drop table_for(JournalEntry)
  end
end
