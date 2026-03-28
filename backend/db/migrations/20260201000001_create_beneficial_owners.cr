class CreateBeneficialOwners::V20260201000001 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(BeneficialOwner) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade

      add first_name_bg : String          # Име на кирилица
      add last_name_bg : String           # Фамилия на кирилица
      add egn : String?                   # ЕГН (за български граждани)
      add first_name_latin : String?      # First name in Latin
      add last_name_latin : String?       # Last name in Latin
      add country : String, default: "BG" # ISO country code
      add ownership_percentage : Float64? # Процент на притежание
      add is_active : Bool, default: true
    end

  end

  def rollback
    drop table_for(BeneficialOwner)
  end
end
