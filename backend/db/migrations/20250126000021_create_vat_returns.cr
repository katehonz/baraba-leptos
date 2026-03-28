class CreateVatReturns::V20250126000021 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(VatReturn) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add period_year : Int32
      add period_month : Int32
      add status : String, default: "draft"
      add result_json : String?
    end

    create_index :vat_returns, [:company_id, :period_year, :period_month], unique: true
  end

  def rollback
    drop table_for(VatReturn)
  end
end
