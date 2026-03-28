class CreateCombinedNomenclatures::V20250126000026 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(CombinedNomenclature) do
      primary_key id : Int64
      add_timestamps
      add cn_code : String
      add description : String
      add primary_unit : String?
      add supplementary_unit : String?
      add level : Int32
      add year : Int32
      add parent_code : String?
      add is_header : Bool, default: false
    end

    create_index :combined_nomenclatures, [:cn_code, :year], unique: true
    create_index :combined_nomenclatures, [:year, :level]
  end

  def rollback
    drop table_for(CombinedNomenclature)
  end
end
