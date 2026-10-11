class CreateDocuments::V20250126000012 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(Document) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add_belongs_to created_by : User?, on_delete: :nullify
      add title : String
      add description : String?
      add file_url : String?
      add file_type : String?
      add file_size : Int64?
      add status : String, default: "pending"
    end

    create_index :documents, [:company_id, :status]
  end

  def rollback
    drop table_for(Document)
  end
end
