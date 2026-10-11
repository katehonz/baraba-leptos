class AddFixedAssetLifecycleFields::V20260823000001 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(FixedAsset) do
      add location : String?
      add conserved_from : Time?
      add conserved_to : Time?
      add disposed_at : Time?
      add disposed_reason : String?
    end
  end

  def rollback
    alter table_for(FixedAsset) do
      remove :location
      remove :conserved_from
      remove :conserved_to
      remove :disposed_at
      remove :disposed_reason
    end
  end
end
