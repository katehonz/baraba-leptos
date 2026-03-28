class AddMissingCompanyColumns::V20250126000046 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(Company) do
      add eik : String, default: ""
      add city : String?
      add country : String?
      add post_code : String?
      add phone : String?
      add email : String?
      add website : String?
      add manager_name : String?
      add manager_eik : String?
      add manager_egn : String?
      add accountant_name : String?
      add accountant_egn : String?
      add tax_authority : String?
      add nap_office : String?
      add inventory_valuation_method : String, default: "FIFO"
      add is_vat_registered : Bool, default: false
      add vat_period : String, default: "monthly"
      add currency : String, default: "EUR"
      add fiscal_year_start_month : Int32, default: 1
      add cash_account_id : Int64?
      add bank_account_id : Int64?
      add customers_account_id : Int64?
      add suppliers_account_id : Int64?
      add revenues_account_id : Int64?
      add expenses_account_id : Int64?
      add delivery_buffer_account_id : Int64?
      add vat_receivable_account_id : Int64?
      add vat_payable_account_id : Int64?
      add invoice_next_number : Int64, default: 1
      add representative_type : String?
      add_belongs_to owner : User?, on_delete: :nullify
    end

    create_index :companies, [:eik], unique: true
  end

  def rollback
    drop_index :companies, [:eik]

    alter table_for(Company) do
      remove :eik
      remove :city
      remove :country
      remove :post_code
      remove :phone
      remove :email
      remove :website
      remove :manager_name
      remove :manager_eik
      remove :manager_egn
      remove :accountant_name
      remove :accountant_egn
      remove :tax_authority
      remove :nap_office
      remove :inventory_valuation_method
      remove :is_vat_registered
      remove :vat_period
      remove :currency
      remove :fiscal_year_start_month
      remove :cash_account_id
      remove :bank_account_id
      remove :customers_account_id
      remove :suppliers_account_id
      remove :revenues_account_id
      remove :expenses_account_id
      remove :delivery_buffer_account_id
      remove :vat_receivable_account_id
      remove :vat_payable_account_id
      remove :invoice_next_number
      remove :representative_type
      remove :owner_id
    end
  end
end
