class CreateBankMatchingRules::V20260911000001 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(BankMatchingRule) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add_belongs_to account : Account, on_delete: :restrict
      add_belongs_to counterpart : Counterpart?, on_delete: :nullify
      add name : String
      add keywords : String
      add direction : String, default: "any" # debit (out), credit (in), any
      add priority : Int32, default: 0
      add is_active : Bool, default: true
    end

    create_index :bank_matching_rules, [:company_id, :priority]
  end

  def rollback
    drop table_for(BankMatchingRule)
  end
end
