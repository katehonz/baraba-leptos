class AddCounterpartIdToJournalEntries::V20260129185402 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(JournalEntry) do
      add counterpart_id : Int64?
    end
  end

  def rollback
    alter table_for(JournalEntry) do
      remove :counterpart_id
    end
  end
end
