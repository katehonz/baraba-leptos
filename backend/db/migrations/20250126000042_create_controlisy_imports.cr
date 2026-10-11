class CreateControlisyImports::V20250126000042 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(ControlisyImport) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add filename : String
      add import_type : String                    # "purchases", "sales", "bank"
      add status : String, default: "pending"    # "pending", "processing", "completed", "failed"
      add documents_count : Int32, default: 0
      add contractors_count : Int32, default: 0
      add journal_entries_count : Int32, default: 0
      add error_message : String?
      add imported_at : Time?
    end
  end

  def rollback
    drop table_for(ControlisyImport)
  end
end
