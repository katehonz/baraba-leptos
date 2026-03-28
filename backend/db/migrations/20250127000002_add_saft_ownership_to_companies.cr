class AddSaftOwnershipToCompanies::V20250127000002 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(Company) do
      # SAF-T Header fields
      add tax_accounting_basis : String, default: "A"   # A, BANK, INSURANCE, P
      add tax_entity : String?                          # Branch/division name
      add software_company_name : String?
      add software_id : String?
      add software_version : String?

      # Ownership Structure (S.O.1-S.O.11)
      # 1=Headquarters, 2=Part of a group, 3=Not part of a group, 4=Solo proprietor, 5=Other
      add is_part_of_group : String?

      # Beneficial Owner (действителен собственик)
      add beneficial_owner_first_name_bg : String?
      add beneficial_owner_last_name_bg : String?
      add beneficial_owner_egn : String?                # ЕГН
      add beneficial_owner_first_name_latin : String?
      add beneficial_owner_last_name_latin : String?
      add beneficial_owner_country : String?            # ISO 3166-1

      # Ultimate Parent (крайно предприятие майка)
      add ultimate_owner_name_bg : String?
      add ultimate_owner_uic : String?                  # ЕИК на крайното предприятие
      add ultimate_owner_name_latin : String?
      add ultimate_owner_country : String?              # ISO 3166-1

      # SAF-T company address structure
      add street_name : String?
      add building_number : String?
      add region : String?                              # ISO 3166-2:BG
    end
  end

  def rollback
    alter table_for(Company) do
      remove :tax_accounting_basis
      remove :tax_entity
      remove :software_company_name
      remove :software_id
      remove :software_version
      remove :is_part_of_group
      remove :beneficial_owner_first_name_bg
      remove :beneficial_owner_last_name_bg
      remove :beneficial_owner_egn
      remove :beneficial_owner_first_name_latin
      remove :beneficial_owner_last_name_latin
      remove :beneficial_owner_country
      remove :ultimate_owner_name_bg
      remove :ultimate_owner_uic
      remove :ultimate_owner_name_latin
      remove :ultimate_owner_country
      remove :street_name
      remove :building_number
      remove :region
    end
  end
end
