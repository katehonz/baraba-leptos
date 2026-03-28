class AddBranchCodeToJournalEntries::V20250131000001 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(JournalEntry) do
      add branch_code : String, default: "0001"
    end
  end

  def rollback
    alter table_for(JournalEntry) do
      remove :branch_code
    end
  end
end
