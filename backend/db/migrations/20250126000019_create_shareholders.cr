class CreateShareholders::V20250126000019 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(Shareholder) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add shareholder_type : String
      add name : String
      add identifier : String?
      add address : String?
      add percentage_owned : Float64?
    end

    create_index :shareholders, [:company_id, :identifier], unique: true
  end

  def rollback
    drop table_for(Shareholder)
  end
end
