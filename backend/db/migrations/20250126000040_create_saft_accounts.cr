class CreateSaftAccounts::V20250126000040 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(SaftAccount) do
      primary_key id : Int64
      add_timestamps
      add code : String                    # e.g., "101", "4531"
      add name : String                    # e.g., "Основен капитал, изискващ регистрация"
      add section_code : String?           # e.g., "1" for section 1
      add section_name : String?           # e.g., "Сметки за капитал и заеми"
      add group_code : String?             # e.g., "10" for group 10
      add group_name : String?             # e.g., "Капитал"
    end

    create_index :saft_accounts, [:code], unique: true
  end

  def rollback
    drop table_for(SaftAccount)
  end
end
