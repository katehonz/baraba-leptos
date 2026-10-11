class CreateSaftCashAccountMappings::V20260201110000 < Avram::Migrator::Migration::V1
  def migrate
    # SAF-T Cash Movement Types (based on payment mechanism codes)
    # These are specific to cash and bank account movements
    execute <<-SQL
      CREATE TABLE IF NOT EXISTS saft_cash_movement_types (
        id BIGSERIAL PRIMARY KEY,
        code VARCHAR(10) NOT NULL UNIQUE,
        name_bg VARCHAR(255) NOT NULL,
        name_en VARCHAR(255),
        payment_method_code VARCHAR(10),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
    SQL

    execute <<-SQL
      INSERT INTO saft_cash_movement_types (code, name_bg, name_en, payment_method_code) VALUES
        ('10', 'Плащане в брой - приход', 'Cash receipt', '01'),
        ('11', 'Плащане в брой - разход', 'Cash payment', '01'),
        ('20', 'Плащане с чек - приход', 'Check receipt', '03'),
        ('21', 'Плащане с чек - разход', 'Check payment', '03'),
        ('42', 'Банков превод - приход', 'Bank transfer receipt', '03'),
        ('43', 'Банков превод - разход', 'Bank transfer payment', '03'),
        ('48', 'Плащане с карта - приход', 'Card payment receipt', '03'),
        ('49', 'Плащане с карта - разход', 'Card payment', '03'),
        ('68', 'Онлайн плащане - приход', 'Online payment receipt', '03'),
        ('69', 'Онлайн плащане - разход', 'Online payment', '03'),
        ('97', 'Прихващане', 'Offset', '02'),
        ('98', 'Бартер', 'Barter', '02'),
        ('99', 'Подотчетни лица', 'Accountable persons', '02'),
        ('80', 'Вътрешен трансфер', 'Internal transfer', NULL),
        ('90', 'Валутна разлика - положителна', 'FX gain', NULL),
        ('91', 'Валутна разлика - отрицателна', 'FX loss', NULL),
        ('100', 'Други парични движения', 'Other cash movements', NULL)
      ON CONFLICT (code) DO NOTHING
    SQL

    # SAF-T Cash Movement Account Mappings
    create table_for(SaftCashAccountMapping) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade

      add cash_movement_type : String    # SAF-T cash movement type code
      add debit_account : String         # Debit account pattern
      add credit_account : String        # Credit account pattern
      add debit_analytical : String?     # Optional analytical debit
      add credit_analytical : String?    # Optional analytical credit
      add description : String?          # Description
      add is_active : Bool, default: true
    end

    # Index for quick lookup
    create_index :saft_cash_account_mappings, [:company_id, :cash_movement_type]
    create_index :saft_cash_account_mappings, [:company_id, :debit_account, :credit_account]
  end

  def rollback
    drop table_for(SaftCashAccountMapping)
    execute "DROP TABLE IF EXISTS saft_cash_movement_types"
  end
end
