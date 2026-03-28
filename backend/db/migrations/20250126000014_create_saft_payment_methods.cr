class CreateSaftPaymentMethods::V20250126000014 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(SaftPaymentMethod) do
      primary_key id : Int64
      add_timestamps
      add method_code : String
      add mechanism_code : String
      add name : String
      add description : String?
    end

    create_index :saft_payment_methods, [:mechanism_code], unique: true
  end

  def rollback
    drop table_for(SaftPaymentMethod)
  end
end
