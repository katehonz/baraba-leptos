class RemoveBranchCodeFromVatJournalEntries::V20250131000002 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(VatJournalEntry) do
      remove :branch_code
    end
  end

  def rollback
    alter table_for(VatJournalEntry) do
      add branch_code : String, default: "0001"
    end
  end
end
