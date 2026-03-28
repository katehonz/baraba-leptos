class CreateSaftInvoiceTypes::V20250126000013 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(SaftInvoiceType) do
      primary_key id : Int64
      add_timestamps
      add code : String
      add name : String
      add description : String?
    end

    create_index :saft_invoice_types, [:code], unique: true
  end

  def rollback
    drop table_for(SaftInvoiceType)
  end
end
