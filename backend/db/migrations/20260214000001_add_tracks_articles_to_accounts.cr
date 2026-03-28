class AddTracksArticlesToAccounts::V20260214000001 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(Account) do
      add tracks_articles : Bool, default: false
    end
  end

  def rollback
    alter table_for(Account) do
      remove :tracks_articles
    end
  end
end
