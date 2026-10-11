class AddEikToCounterparts::V20250126000043 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(Counterpart) do
      add eik : String?
      add is_active : Bool, default: true
    end

    create_index :counterparts, [:company_id, :eik], unique: false
  end

  def rollback
    alter table_for(Counterpart) do
      remove :eik
      remove :is_active
    end
  end
end
