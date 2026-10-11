class CreateVatRates::V20250126000004 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(VatRate) do
      primary_key id : Int64
      add_timestamps
      add name : String
      add percentage : Float64
      add code : String
      add saft_tax_type : String?
      add saft_tax_code : String?
      add effective_from : Time?
      add effective_to : Time?
      add is_active : Bool, default: true
    end

    create_index :vat_rates, [:code], unique: true
  end

  def rollback
    drop table_for(VatRate)
  end
end
