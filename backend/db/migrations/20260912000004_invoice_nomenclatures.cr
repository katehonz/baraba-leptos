class InvoiceNomenclatures::V20260912000004 < Avram::Migrator::Migration::V1
  def migrate
    execute <<-SQL
      CREATE TABLE IF NOT EXISTS vat_exemption_reasons (
        id BIGSERIAL PRIMARY KEY,
        reason_code VARCHAR(64) NOT NULL UNIQUE,
        reason_name VARCHAR(255) NOT NULL,
        reason_name_en VARCHAR(255),
        legal_basis TEXT,
        legal_basis_en TEXT,
        ubl_category_code VARCHAR(8),
        ubl_exemption_code VARCHAR(64),
        is_active BOOLEAN NOT NULL DEFAULT true,
        sort_order INTEGER NOT NULL DEFAULT 100,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      );
    SQL

    execute <<-SQL
      INSERT INTO vat_exemption_reasons
        (reason_code, reason_name, reason_name_en, legal_basis, legal_basis_en, ubl_category_code, ubl_exemption_code, is_active, sort_order)
      VALUES
        ('VATEX-EU-G','Износ на стоки','Export of goods','чл. 28 от ЗДДС','Art. 28 VAT Act','G','vatex-eu-g',true,1),
        ('VATEX-EU-IC','Вътреобщностна доставка на стоки','Intra-Community supply of goods','чл. 7, ал. 1 от ЗДДС','Art. 7(1) VAT Act','K','vatex-eu-ic',true,2),
        ('VATEX-EU-O','Обратно начисляване на ДДС','Reverse charge','чл. 82, ал. 2-5 от ЗДДС','Art. 82(2-5) VAT Act','O','vatex-eu-o',true,3),
        ('VATEX-EU-AE','Специален режим на обратно начисляване','VAT Reverse Charge','чл. 163а от ЗДДС','Art. 163a VAT Act','AE','vatex-eu-ae',true,4),
        ('VATEX-EU-E-39','Освободена доставка - здравеопазване','Exempt - healthcare','чл. 39 от ЗДДС','Art. 39 VAT Act','E','vatex-eu-e',true,10),
        ('VATEX-EU-E-40','Освободена доставка - социални услуги','Exempt - social services','чл. 40 от ЗДДС','Art. 40 VAT Act','E','vatex-eu-e',true,11),
        ('VATEX-EU-E-41','Освободена доставка - образование','Exempt - education','чл. 41 от ЗДДС','Art. 41 VAT Act','E','vatex-eu-e',true,12),
        ('VATEX-EU-E-42','Освободена доставка - културни услуги','Exempt - cultural services','чл. 42 от ЗДДС','Art. 42 VAT Act','E','vatex-eu-e',true,13),
        ('VATEX-EU-E-44','Освободена доставка - финансови услуги','Exempt - financial services','чл. 44 от ЗДДС','Art. 44 VAT Act','E','vatex-eu-e',true,14),
        ('VATEX-EU-E-45','Освободена доставка - застраховки','Exempt - insurance','чл. 45 от ЗДДС','Art. 45 VAT Act','E','vatex-eu-e',true,15),
        ('VATEX-EU-E-46','Освободена доставка - недвижими имоти','Exempt - immovable property','чл. 45-46 от ЗДДС','Art. 45-46 VAT Act','E','vatex-eu-e',true,16),
        ('VATEX-EU-E-47','Освободена доставка - хазартни услуги','Exempt - gambling','чл. 47 от ЗДДС','Art. 47 VAT Act','E','vatex-eu-e',true,17),
        ('VATEX-EU-Z-29','Нулева ставка - международен транспорт','Zero rate - international transport','чл. 29 от ЗДДС','Art. 29 VAT Act','Z','vatex-eu-z',true,20),
        ('VATEX-EU-Z-30','Нулева ставка - услуги за плавателни съдове','Zero rate - maritime services','чл. 30 от ЗДДС','Art. 30 VAT Act','Z','vatex-eu-z',true,21),
        ('VATEX-EU-Z-31','Нулева ставка - услуги за въздухоплавателни средства','Zero rate - aircraft services','чл. 31 от ЗДДС','Art. 31 VAT Act','Z','vatex-eu-z',true,22),
        ('VATEX-EU-Z-32','Нулева ставка - транзитни стоки','Zero rate - transit goods','чл. 32 от ЗДДС','Art. 32 VAT Act','Z','vatex-eu-z',true,23),
        ('VATEX-EU-Z-33','Нулева ставка - митнически режими','Zero rate - customs regimes','чл. 33 от ЗДДС','Art. 33 VAT Act','Z','vatex-eu-z',true,24),
        ('VATEX-EU-Z-173','Нулева ставка - дипломатически мисии','Zero rate - diplomatic missions','чл. 173 от ЗДДС','Art. 173 VAT Act','Z','vatex-eu-z',true,25),
        ('VATEX-EU-S-21','Услуги към лица извън ЕС','Services to non-EU persons','чл. 21, ал. 2 от ЗДДС','Art. 21(2) VAT Act','O','vatex-eu-o',true,30),
        ('VATEX-EU-S-22','Услуги, свързани с недвижим имот извън БГ','Services related to immovable property outside BG','чл. 21, ал. 4, т. 1 от ЗДДС','Art. 21(4)(1) VAT Act','O','vatex-eu-o',true,31),
        ('VATEX-EU-SM','Освободена доставка - малко предприятие','Small business exemption','чл. 113 от ЗДДС','Art. 113 VAT Act','E','vatex-eu-e',true,40)
      ON CONFLICT (reason_code) DO NOTHING;
    SQL

    execute <<-SQL
      CREATE TABLE IF NOT EXISTS document_type_codes (
        id BIGSERIAL PRIMARY KEY,
        code VARCHAR(10) NOT NULL UNIQUE,
        name VARCHAR(255) NOT NULL,
        name_en VARCHAR(255) NOT NULL,
        applies_to VARCHAR(32) NOT NULL,
        is_common BOOLEAN NOT NULL DEFAULT false,
        sort_order INTEGER NOT NULL DEFAULT 100,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
      SQL

    execute <<-SQL
      INSERT INTO document_type_codes (code, name, name_en, applies_to, is_common, sort_order) VALUES
        ('380','Търговска фактура','Commercial invoice','INVOICE',true,1),
        ('381','Кредитно известие','Credit note','CREDIT_NOTE',true,2),
        ('383','Дебитно известие','Debit note','INVOICE',true,3),
        ('384','Коригираща фактура','Corrected invoice','INVOICE',true,4),
        ('386','Авансова фактура','Prepayment invoice','INVOICE',true,5),
        ('388','Данъчна фактура','Tax invoice','INVOICE',true,7),
        ('389','Самофактура','Self-billed invoice','INVOICE',true,6)
      ON CONFLICT (code) DO NOTHING;
    SQL

    execute <<-SQL
      CREATE TABLE IF NOT EXISTS warehouse_document_types (
        id BIGSERIAL PRIMARY KEY,
        code VARCHAR(32) NOT NULL UNIQUE,
        name_bg VARCHAR(255) NOT NULL,
        saft_movement_type VARCHAR(8) NOT NULL,
        needs_from BOOLEAN NOT NULL DEFAULT false,
        needs_to BOOLEAN NOT NULL DEFAULT false,
        default_is_incoming BOOLEAN,
        system_only BOOLEAN NOT NULL DEFAULT false,
        sort_order INTEGER NOT NULL DEFAULT 100,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
      SQL

    execute <<-SQL
      INSERT INTO warehouse_document_types
        (code, name_bg, saft_movement_type, needs_from, needs_to, default_is_incoming, system_only, sort_order)
      VALUES
        ('RECEIPT','Складова разписка / приход','10',false,true,true,false,10),
        ('PROD_RECEIPT','Произведена продукция','20',false,true,true,false,20),
        ('SALES_ISSUE','Продажба (системно)','30',true,false,false,true,30),
        ('SALES_RETURN','Връщане от клиент (системно)','40',false,true,true,true,40),
        ('PURCHASE_RETURN','Връщане към доставчик (системно)','50',true,false,false,true,50),
        ('ISSUE','Вътрешен разход','70',true,false,false,false,70),
        ('PROD_ISSUE','Влагане в производство','70',true,false,false,false,71),
        ('TRANSFER','Вътрешен трансфер','80',true,true,NULL,false,80),
        ('INV_PLUS','Инвентаризация — излишък','110',false,true,true,false,110),
        ('INV_MINUS','Инвентаризация — липса','120',true,false,false,false,120),
        ('WRITE_OFF','Брак','160',true,false,false,false,160),
        ('EXPIRY','Изтекъл срок','170',true,false,false,false,170),
        ('OTHER','Друго движение','180',false,false,NULL,false,180)
      ON CONFLICT (code) DO NOTHING;
    SQL

    execute <<-SQL
      CREATE TABLE IF NOT EXISTS nap_document_types (
        id BIGSERIAL PRIMARY KEY,
        code VARCHAR(2) NOT NULL,
        journal VARCHAR(16) NOT NULL,
        name_bg VARCHAR(255) NOT NULL,
        saft_invoice_type VARCHAR(8),
        is_tax BOOLEAN NOT NULL DEFAULT true,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        UNIQUE (code, journal)
      )
      SQL

    execute <<-SQL
      INSERT INTO nap_document_types (code, journal, name_bg, saft_invoice_type, is_tax) VALUES
        ('01','sales','Фактура','1',true),
        ('02','sales','Дебитно известие','2',true),
        ('03','sales','Кредитно известие','3',true),
        ('01','purchase','Фактура','1',true),
        ('02','purchase','Дебитно известие','2',true),
        ('03','purchase','Кредитно известие','3',true),
        ('09','sales','Протокол','9',true),
        ('09','purchase','Протокол','9',true)
      ON CONFLICT (code, journal) DO NOTHING;
    SQL
  end

  def rollback
    execute "DROP TABLE IF EXISTS nap_document_types"
    execute "DROP TABLE IF EXISTS warehouse_document_types"
    execute "DROP TABLE IF EXISTS document_type_codes"
    execute "DROP TABLE IF EXISTS vat_exemption_reasons"
  end
end
