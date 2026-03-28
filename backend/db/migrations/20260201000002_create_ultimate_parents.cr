class CreateUltimateParents::V20260201000002 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(UltimateParent) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade

      add name_bg : String                # Наименование на кирилица
      add name_latin : String?            # Name in Latin
      add uic : String?                   # ЕИК/UIC
      add country : String, default: "BG" # ISO country code
      add is_active : Bool, default: true
    end

  end

  def rollback
    drop table_for(UltimateParent)
  end
end
