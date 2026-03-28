class AddTimestampsToSaftAssetAccountMappings::V20260130000004 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(SaftAssetAccountMapping) do
      add created_at : Time, default: Time.utc
      add updated_at : Time, default: Time.utc
    end
  end

  def rollback
    alter table_for(SaftAssetAccountMapping) do
      remove :created_at
      remove :updated_at
    end
  end
end
