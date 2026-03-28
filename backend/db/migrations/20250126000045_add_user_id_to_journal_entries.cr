class AddUserIdToJournalEntries::V20250126000045 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(JournalEntry) do
      add_belongs_to user : User?, on_delete: :nullify
    end
  end

  def rollback
    alter table_for(JournalEntry) do
      remove_belongs_to :user
    end
  end
end
