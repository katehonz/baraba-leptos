class AddVatSubmissionFields::V20250129000004 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(Company) do
      # Пълномощник (Authorized Person) for VAT submission
      add authorized_person_name : String?, default: nil
      add authorized_person_egn : String?, default: nil

      # Branch/Division number for companies with multiple branches
      add vat_branch_number : Int32?, default: nil
    end
  end

  def rollback
    alter table_for(Company) do
      remove :authorized_person_name
      remove :authorized_person_egn
      remove :vat_branch_number
    end
  end
end
