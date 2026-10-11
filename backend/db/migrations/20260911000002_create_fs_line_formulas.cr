class CreateFsLineFormulas::V20260911000002 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(FsLineFormula) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add statement : String
      add line_code : String
      add formula : String, default: ""
    end

    create_index :fs_line_formulas, [:company_id, :statement, :line_code], unique: true
  end

  def rollback
    drop table_for(FsLineFormula)
  end
end
