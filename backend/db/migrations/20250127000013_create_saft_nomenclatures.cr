class CreateSaftNomenclatures::V20250127000013 < Avram::Migrator::Migration::V1
  def migrate
    # SAF-T Tax Regimes
    execute <<-SQL
      CREATE TABLE IF NOT EXISTS saft_tax_regimes (
        id BIGSERIAL PRIMARY KEY,
        code VARCHAR(10) NOT NULL UNIQUE,
        name VARCHAR(255) NOT NULL,
        description VARCHAR(500),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
    SQL

    execute <<-SQL
      INSERT INTO saft_tax_regimes (code, name, description) VALUES
        ('100010', 'Данъчно задължено лице, регистрирано за целите на ДДС', 'Taxable person registered for VAT'),
        ('100020', 'Всяко друго данъчно задължено лице', 'Any other taxable person'),
        ('100030', 'Данъчно незадължено лице', 'Non-taxable person')
      ON CONFLICT (code) DO NOTHING
    SQL

    # SAF-T Payment Methods
    execute <<-SQL
      CREATE TABLE IF NOT EXISTS saft_payment_methods (
        id BIGSERIAL PRIMARY KEY,
        method_code VARCHAR(10) NOT NULL,
        mechanism_code VARCHAR(10) NOT NULL UNIQUE,
        name VARCHAR(255) NOT NULL,
        description VARCHAR(500),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
    SQL

    execute <<-SQL
      INSERT INTO saft_payment_methods (method_code, mechanism_code, name, description) VALUES
        ('01', '10', 'Пари в брой', 'Cash'),
        ('02', '97', 'Прихващане между контрагенти', 'Offset between counterparts'),
        ('02', '98', 'Бартер', 'Barter'),
        ('02', '99', 'Подотчетни лица', 'Accountable persons'),
        ('03', '20', 'С чек', 'By check'),
        ('03', '42', 'Плащане по банкова сметка', 'Bank account payment'),
        ('03', '48', 'Банкова карта', 'Bank card'),
        ('03', '68', 'Услуги за онлайн плащане', 'Online payment services'),
        ('03', '30', 'Ваучер', 'Voucher')
      ON CONFLICT (mechanism_code) DO NOTHING
    SQL

    # SAF-T Stock Movement Types
    execute <<-SQL
      CREATE TABLE IF NOT EXISTS saft_stock_movement_types (
        id BIGSERIAL PRIMARY KEY,
        code VARCHAR(10) NOT NULL UNIQUE,
        name_bg VARCHAR(255) NOT NULL,
        name_en VARCHAR(255),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
    SQL

    execute <<-SQL
      INSERT INTO saft_stock_movement_types (code, name_bg, name_en) VALUES
        ('10', 'Покупка', 'Purchase'),
        ('20', 'Материални запаси от производство', 'Inventory from Production'),
        ('30', 'Продажба', 'Sale'),
        ('40', 'Връщане на продадени продукти', 'Return of Sold Products'),
        ('50', 'Връщане на закупени продукти', 'Return of Purchased Products'),
        ('60', 'Получени отстъпки в натура', 'Discounts Received in Kind'),
        ('65', 'Предоставени отстъпки в натура', 'Discounts Given in Kind'),
        ('70', 'Материални запаси за производство', 'Inventory for Production'),
        ('80', 'Вътрешен трансфер', 'Internal Transfer'),
        ('90', 'Последващи разходи', 'Subsequent Costs Capitalized'),
        ('100', 'Положителна ценова разлика', 'Positive Price Difference'),
        ('101', 'Отрицателна ценова разлика', 'Negative Price Difference'),
        ('110', 'Положителна корекция от инвентаризация', 'Positive Inventory Adjustment'),
        ('120', 'Отрицателна корекция от инвентаризация', 'Negative Inventory Adjustment'),
        ('130', 'Увеличение от преоценка', 'Increase from Revaluation'),
        ('140', 'Намаление от преоценка', 'Decrease from Revaluation'),
        ('150', 'Безвъзмездно предоставени', 'Gratuitous Transfer'),
        ('160', 'Брак', 'Scrap'),
        ('170', 'Изтекъл срок на годност', 'Expired'),
        ('180', 'Други движения', 'Other Movements')
      ON CONFLICT (code) DO NOTHING
    SQL

    # SAF-T Asset Movement Types
    execute <<-SQL
      CREATE TABLE IF NOT EXISTS saft_asset_movement_types (
        id BIGSERIAL PRIMARY KEY,
        code VARCHAR(10) NOT NULL UNIQUE,
        name_bg VARCHAR(255) NOT NULL,
        name_en VARCHAR(255),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
    SQL

    execute <<-SQL
      INSERT INTO saft_asset_movement_types (code, name_bg, name_en) VALUES
        ('10', 'Придобиване', 'Acquisition'),
        ('20', 'Продажба', 'Sale'),
        ('30', 'Амортизация', 'Depreciation'),
        ('40', 'Вътрешен трансфер', 'Internal Transfer'),
        ('50', 'Брак', 'Scrapping'),
        ('60', 'Преоценка (отрицателна)', 'Revaluation (negative)'),
        ('70', 'Преоценка (положителна)', 'Revaluation (positive)'),
        ('80', 'Излишък (инвентаризация)', 'Surplus (inventory)'),
        ('90', 'Липса (инвентаризация)', 'Shortage (inventory)'),
        ('100', 'Обезценка', 'Impairment'),
        ('110', 'Сторно на обезценка', 'Impairment Reversal'),
        ('120', 'Безвъзмездно предоставени', 'Gratuitous Transfer'),
        ('130', 'Други транзакции', 'Other Transactions')
      ON CONFLICT (code) DO NOTHING
    SQL

    # SAF-T Invoice Types
    execute <<-SQL
      CREATE TABLE IF NOT EXISTS saft_invoice_types (
        id BIGSERIAL PRIMARY KEY,
        code VARCHAR(10) NOT NULL UNIQUE,
        name VARCHAR(255) NOT NULL,
        description VARCHAR(500),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
    SQL

    execute <<-SQL
      INSERT INTO saft_invoice_types (code, name, description) VALUES
        ('1', 'Фактура', 'Invoice'),
        ('2', 'Дебитно известие', 'Debit Note'),
        ('3', 'Кредитно известие', 'Credit Note'),
        ('4', 'Регистър на стоки - изпратени', 'Call-off stock register - sent'),
        ('5', 'Регистър на стоки - получени', 'Call-off stock register - received'),
        ('7', 'Митническа декларация', 'Customs Declaration'),
        ('9', 'Протокол или друг документ', 'Protocol or other document'),
        ('11', 'Фактура - касова отчетност', 'Invoice - cash accounting'),
        ('12', 'Дебитно известие – касова отчетност', 'Debit note - cash accounting'),
        ('13', 'Кредитно известие – касова отчетност', 'Credit note - cash accounting'),
        ('23', 'Кредитно известие по чл. 126б, ал. 1', 'Credit note Art. 126b (1)'),
        ('29', 'Протокол по чл. 126б, ал. 2 и 7', 'Protocol Art. 126b (2) and (7)'),
        ('81', 'Отчет за извършените продажби', 'Sales report'),
        ('82', 'Отчет - специален ред на облагане', 'Sales report - special taxation'),
        ('91', 'Протокол по чл. 151в, ал. 3', 'Protocol Art. 151v (3)'),
        ('92', 'Протокол по чл. 151г, ал. 8', 'Protocol Art. 151g (8)'),
        ('93', 'Протокол по чл. 151в, ал. 7 - без режим', 'Protocol Art. 151v (7) - non-special'),
        ('94', 'Протокол по чл. 151в, ал. 7 - с режим', 'Protocol Art. 151v (7) - special'),
        ('95', 'Протокол за безвъзмездни храни', 'Protocol for gratuitous food supplies')
      ON CONFLICT (code) DO NOTHING
    SQL

    # SAF-T Product Types
    execute <<-SQL
      CREATE TABLE IF NOT EXISTS saft_product_types (
        id BIGSERIAL PRIMARY KEY,
        code VARCHAR(10) NOT NULL UNIQUE,
        name VARCHAR(255) NOT NULL,
        description VARCHAR(500),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
    SQL

    execute <<-SQL
      INSERT INTO saft_product_types (code, name, description) VALUES
        ('10', 'Материали', 'Materials'),
        ('20', 'Продукция', 'Finished goods / Production'),
        ('30', 'Стоки', 'Goods / Merchandise'),
        ('40', 'Незавършено производство', 'Work in Progress (WIP)'),
        ('50', 'Инвестиция в материален запас', 'Investment in Inventory')
      ON CONFLICT (code) DO NOTHING
    SQL

    # SAF-T Units of Measure (UN/ECE Recommendation 20)
    execute <<-SQL
      CREATE TABLE IF NOT EXISTS saft_units_of_measure (
        id BIGSERIAL PRIMARY KEY,
        code VARCHAR(10) NOT NULL UNIQUE,
        name_en VARCHAR(255) NOT NULL,
        name_bg VARCHAR(255) NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
    SQL

    execute <<-SQL
      INSERT INTO saft_units_of_measure (code, name_en, name_bg) VALUES
        ('C62', 'one', 'брой'),
        ('KGM', 'kilogram', 'килограм'),
        ('GRM', 'gram', 'грам'),
        ('TNE', 'tonne', 'тон'),
        ('LTR', 'litre', 'литър'),
        ('MLT', 'millilitre', 'милилитър'),
        ('MTR', 'metre', 'метър'),
        ('CMT', 'centimetre', 'сантиметър'),
        ('MMT', 'millimetre', 'милиметър'),
        ('MTK', 'square metre', 'квадратен метър'),
        ('MTQ', 'cubic metre', 'кубичен метър'),
        ('LM', 'linear metre', 'линеен метър'),
        ('PR', 'pair', 'чифт'),
        ('SET', 'set', 'комплект'),
        ('PCE', 'piece', 'парче'),
        ('BX', 'box', 'кутия'),
        ('CT', 'carton', 'картон'),
        ('PK', 'pack', 'пакет'),
        ('BG', 'bag', 'торба'),
        ('RL', 'roll', 'ролка'),
        ('BTL', 'bottle', 'бутилка'),
        ('CAN', 'can', 'консерва'),
        ('DZN', 'dozen', 'дузина'),
        ('HUR', 'hour', 'час'),
        ('DAY', 'day', 'ден'),
        ('MON', 'month', 'месец'),
        ('ANN', 'year', 'година'),
        ('MIN', 'minute', 'минута'),
        ('SEC', 'second', 'секунда'),
        ('KWH', 'kilowatt hour', 'киловатчас'),
        ('MWH', 'megawatt hour', 'мегаватчас'),
        ('KMT', 'kilometre', 'километър'),
        ('HAR', 'hectare', 'хектар'),
        ('DAA', 'decare', 'декар')
      ON CONFLICT (code) DO NOTHING
    SQL
  end

  def rollback
    execute "DROP TABLE IF EXISTS saft_tax_regimes"
    execute "DROP TABLE IF EXISTS saft_payment_methods"
    execute "DROP TABLE IF EXISTS saft_stock_movement_types"
    execute "DROP TABLE IF EXISTS saft_asset_movement_types"
    execute "DROP TABLE IF EXISTS saft_invoice_types"
    execute "DROP TABLE IF EXISTS saft_product_types"
    execute "DROP TABLE IF EXISTS saft_units_of_measure"
  end
end
