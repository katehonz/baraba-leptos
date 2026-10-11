class CreateSaftExtendedNomenclatures::V20260130000001 < Avram::Migrator::Migration::V1
  def migrate
    # SAF-T Bulgarian Regions (ISO 3166-2:BG)
    execute <<-SQL
      CREATE TABLE IF NOT EXISTS saft_regions (
        id BIGSERIAL PRIMARY KEY,
        code VARCHAR(10) NOT NULL UNIQUE,
        name VARCHAR(255) NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
    SQL

    execute <<-SQL
      INSERT INTO saft_regions (code, name) VALUES
        ('BG-01', 'Благоевград'),
        ('BG-02', 'Бургас'),
        ('BG-03', 'Варна'),
        ('BG-04', 'Велико Търново'),
        ('BG-05', 'Видин'),
        ('BG-06', 'Враца'),
        ('BG-07', 'Габрово'),
        ('BG-08', 'Добрич'),
        ('BG-09', 'Кърджали'),
        ('BG-10', 'Кюстендил'),
        ('BG-11', 'Ловеч'),
        ('BG-12', 'Монтана'),
        ('BG-13', 'Пазарджик'),
        ('BG-14', 'Перник'),
        ('BG-15', 'Плевен'),
        ('BG-16', 'Пловдив'),
        ('BG-17', 'Разград'),
        ('BG-18', 'Русе'),
        ('BG-19', 'Силистра'),
        ('BG-20', 'Сливен'),
        ('BG-21', 'Смолян'),
        ('BG-22', 'София'),
        ('BG-23', 'Софийска област'),
        ('BG-24', 'Стара Загора'),
        ('BG-25', 'Търговище'),
        ('BG-26', 'Хасково'),
        ('BG-27', 'Шумен'),
        ('BG-28', 'Ямбол')
      ON CONFLICT (code) DO NOTHING
    SQL

    # SAF-T IBAN Formats (ISO 13616)
    execute <<-SQL
      CREATE TABLE IF NOT EXISTS saft_iban_formats (
        id BIGSERIAL PRIMARY KEY,
        country VARCHAR(100) NOT NULL,
        length INT NOT NULL,
        bban_format VARCHAR(100),
        iban_fields VARCHAR(255),
        comments TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
    SQL

    execute <<-SQL
      INSERT INTO saft_iban_formats (country, length, bban_format, iban_fields, comments) VALUES
        ('Albania', 28, '8n,16c', 'ALkk bbbs sssx cccc cccc cccc cccc', 'b=bank, s=branch, x=check, c=account'),
        ('Andorra', 24, '8n,12c', 'ADkk bbbb ssss cccc cccc cccc', 'b=bank, s=branch, c=account'),
        ('Austria', 20, '16n', 'ATkk bbbb bccc cccc cccc', 'b=bank, c=account'),
        ('Azerbaijan', 28, '4c,20n', 'AZkk bbbb cccc cccc cccc cccc cccc', 'b=bank, c=account'),
        ('Bahrain', 22, '4a,14c', 'BHkk bbbb cccc cccc cccc cc', 'b=bank, c=account'),
        ('Belarus', 28, '4c,4n,16c', 'BYkk bbbb aaaa cccc cccc cccc cccc', 'b=bank, a=balance, c=account'),
        ('Belgium', 16, '12n', 'BEkk bbbc cccc ccxx', 'b=bank, c=account, x=check'),
        ('Bosnia and Herzegovina', 20, '16n', 'BAkk bbbs sscc cccc ccxx', 'k=39, b=bank, s=branch, c=account, x=check'),
        ('Brazil', 29, '23n,1a,1c', 'BRkk bbbb bbbb ssss sccc cccc ccct n', 'b=bank, s=branch, c=account, t=type, n=owner'),
        ('Bulgaria', 22, '4a,6n,8c', 'BGkk bbbb ssss ttcc cccc cc', 'b=BIC bank, s=BAE branch, t=account type, c=account'),
        ('Croatia', 21, '17n', 'HRkk bbbb bbbc cccc cccc c', 'b=bank, c=account'),
        ('Cyprus', 28, '8n,16c', 'CYkk bbbs ssss cccc cccc cccc cccc', 'b=bank, s=branch, c=account'),
        ('Czech Republic', 24, '20n', 'CZkk bbbb ssss sscc cccc cccc', 'b=bank, s=prefix, c=account'),
        ('Denmark', 18, '14n', 'DKkk bbbb cccc cccc cc', 'b=bank, c=account'),
        ('Estonia', 20, '16n', 'EEkk bbss cccc cccc cccx', 'b=bank, s=branch, c=account, x=check'),
        ('Finland', 18, '14n', 'FIkk bbbb bbcc cccc cx', 'b=bank/branch, c=account, x=check'),
        ('France', 27, '10n,11c,2n', 'FRkk bbbb bsss sscc cccc cccc cxx', 'b=bank, s=branch, c=account, x=check'),
        ('Georgia', 22, '2c,16n', 'GEkk bbcc cccc cccc cccc cc', 'b=bank, c=account'),
        ('Germany', 22, '18n', 'DEkk bbbb bbbb cccc cccc cc', 'b=BLZ, c=account'),
        ('Gibraltar', 23, '4a,15c', 'GIkk bbbb cccc cccc cccc ccc', 'b=BIC bank, c=account'),
        ('Greece', 27, '7n,16c', 'GRkk bbbs sssc cccc cccc cccc ccc', 'b=bank, s=branch, c=account'),
        ('Hungary', 28, '24n', 'HUkk bbbs sssx cccc cccc cccc cccx', 'b=bank, s=branch, c=account, x=check'),
        ('Iceland', 26, '22n', 'ISkk bbss ttcc cccc iiii iiii ii', 'b=bank, s=branch, t=type, c=account, i=kennitala'),
        ('Ireland', 22, '4c,14n', 'IEkk aaaa bbbb bbcc cccc cc', 'a=BIC, b=sort code, c=account'),
        ('Israel', 23, '19n', 'ILkk bbbs sscc cccc cccc ccc', 'b=bank, s=branch, c=account'),
        ('Italy', 27, '1a,10n,12c', 'ITkk xbbb bbss sssc cccc cccc ccc', 'x=CIN, b=ABI, s=CAB, c=account'),
        ('Jordan', 30, '4a,22n', 'JOkk bbbb ssss cccc cccc cccc cccc cc', 'b=bank, s=branch, c=account'),
        ('Kazakhstan', 20, '3n,13c', 'KZkk bbbc cccc cccc cccc', 'b=bank, c=account'),
        ('Kosovo', 20, '4n,10n,2n', 'XKkk bbbb cccc cccc cccc', 'b=bank, c=account'),
        ('Kuwait', 30, '4a,22c', 'KWkk bbbb cccc cccc cccc cccc cccc cc', 'b=bank, c=account'),
        ('Latvia', 21, '4a,13c', 'LVkk bbbb cccc cccc cccc c', 'b=BIC, c=account'),
        ('Lebanon', 28, '4n,20c', 'LBkk bbbb cccc cccc cccc cccc cccc', 'b=bank, c=account'),
        ('Liechtenstein', 21, '5n,12c', 'LIkk bbbb bccc cccc cccc c', 'b=bank, c=account'),
        ('Lithuania', 20, '16n', 'LTkk bbbb bccc cccc cccc', 'b=bank, c=account'),
        ('Luxembourg', 20, '3n,13c', 'LUkk bbbc cccc cccc cccc', 'b=bank, c=account'),
        ('Malta', 31, '4a,5n,18c', 'MTkk bbbb ssss sccc cccc cccc cccc ccc', 'b=BIC, s=branch, c=account'),
        ('Monaco', 27, '10n,11c,2n', 'MCkk bbbb bsss sscc cccc cccc cxx', 'b=bank, s=branch, c=account, x=check'),
        ('Moldova', 24, '2c,18c', 'MDkk bbcc cccc cccc cccc cccc', 'b=bank, c=account'),
        ('Montenegro', 22, '18n', 'MEkk bbbc cccc cccc cccc xx', 'k=25, b=bank, c=account, x=check'),
        ('Netherlands', 18, '4a,10n', 'NLkk bbbb cccc cccc cc', 'b=BIC, c=account'),
        ('North Macedonia', 19, '3n,10c,2n', 'MKkk bbbc cccc cccc cxx', 'k=07, b=bank, c=account, x=check'),
        ('Norway', 15, '11n', 'NOkk bbbb cccc ccx', 'b=bank, c=account, x=check'),
        ('Pakistan', 24, '4c,16n', 'PKkk bbbb cccc cccc cccc cccc', 'b=bank, c=account'),
        ('Poland', 28, '24n', 'PLkk bbbs sssx cccc cccc cccc cccc', 'b=bank, s=branch, x=check, c=account'),
        ('Portugal', 25, '21n', 'PTkk bbbb ssss cccc cccc cccx x', 'k=50, b=bank, s=branch, c=account, x=check'),
        ('Qatar', 29, '4a,21c', 'QAkk bbbb cccc cccc cccc cccc cccc c', 'b=bank, c=account'),
        ('Romania', 24, '4a,16c', 'ROkk bbbb cccc cccc cccc cccc', 'b=BIC, c=branch+account'),
        ('San Marino', 27, '1a,10n,12c', 'SMkk xbbb bbss sssc cccc cccc ccc', 'x=CIN, b=ABI, s=CAB, c=account'),
        ('Saudi Arabia', 24, '2n,18c', 'SAkk bbcc cccc cccc cccc cccc', 'b=bank, c=account'),
        ('Serbia', 22, '18n', 'RSkk bbbc cccc cccc cccc xx', 'k=35, b=bank, c=account, x=check'),
        ('Slovakia', 24, '20n', 'SKkk bbbb ssss sscc cccc cccc', 'b=bank, s=prefix, c=account'),
        ('Slovenia', 19, '15n', 'SIkk bbss sccc cccc cxx', 'k=56, b=bank, s=branch, c=account, x=check'),
        ('Spain', 24, '20n', 'ESkk bbbb ssss xxcc cccc cccc', 'b=bank, s=branch, x=check, c=account'),
        ('Sweden', 24, '20n', 'SEkk bbbc cccc cccc cccc cccc', 'b=bank, c=account'),
        ('Switzerland', 21, '5n,12c', 'CHkk bbbb bccc cccc cccc c', 'b=bank, c=account'),
        ('Turkey', 26, '5n,17c', 'TRkk bbbb b0cc cccc cccc cccc cc', 'b=bank, 0=reserved, c=account'),
        ('Ukraine', 29, '6n,19c', 'UAkk bbbb bbcc cccc cccc cccc cccc c', 'b=bank, c=account'),
        ('United Arab Emirates', 23, '3n,16n', 'AEkk bbbc cccc cccc cccc ccc', 'b=bank, c=account'),
        ('United Kingdom', 22, '4a,14n', 'GBkk bbbb ssss sscc cccc cc', 'b=BIC, s=sort code, c=account'),
        ('Vatican City', 22, '3n,15n', 'VAkk bbbc cccc cccc cccc cc', 'b=bank, c=account')
      ON CONFLICT DO NOTHING
    SQL

    # Extend saft_units_of_measure with more common units
    execute <<-SQL
      INSERT INTO saft_units_of_measure (code, name_en, name_bg) VALUES
        ('10', 'group', 'група'),
        ('11', 'outfit', 'комплект'),
        ('27', 'ton', 'тон'),
        ('58', 'net kilogram', 'килограм нето'),
        ('2N', 'decibel', 'децибел'),
        ('A11', 'milliampere', 'милиампер'),
        ('A39', 'hectolitre', 'хектолитър'),
        ('A43', 'kilowatt', 'киловат'),
        ('A86', 'gigabyte', 'гигабайт'),
        ('B10', 'bit per second', 'бит за секунда'),
        ('B49', 'kilovolt', 'киловолт'),
        ('C36', 'megabyte', 'мегабайт'),
        ('D63', 'book', 'книга'),
        ('E14', 'kilocalorie', 'килокалория'),
        ('E27', 'dose', 'доза'),
        ('E49', 'working day', 'работен ден'),
        ('FB', 'field', 'поле'),
        ('H87', 'piece', 'брой'),
        ('HTZ', 'hertz', 'херц'),
        ('INH', 'inch', 'инч'),
        ('JOU', 'joule', 'джаул'),
        ('KGS', 'kilogram per second', 'килограм за секунда'),
        ('KHZ', 'kilohertz', 'килохерц'),
        ('KMK', 'square kilometre', 'квадратен километър'),
        ('KPA', 'kilopascal', 'килопаскал'),
        ('KWN', 'kilowatt', 'киловат'),
        ('LEF', 'leaf', 'лист'),
        ('MBR', 'millibar', 'милибар'),
        ('MGM', 'milligram', 'милиграм'),
        ('MHZ', 'megahertz', 'мегахерц'),
        ('MIK', 'square mile', 'квадратна миля'),
        ('MPA', 'megapascal', 'мегапаскал'),
        ('NAR', 'number of articles', 'брой артикули'),
        ('NPR', 'number of pairs', 'брой чифтове'),
        ('NRL', 'number of rolls', 'брой ролки'),
        ('OHM', 'ohm', 'ом'),
        ('P1', 'percent', 'процент'),
        ('PA', 'packet', 'пакет'),
        ('PAL', 'pallet', 'палет'),
        ('PFL', 'proof litre', 'литър доказан'),
        ('RPM', 'revolutions per minute', 'обороти в минута'),
        ('SEC', 'second', 'секунда'),
        ('SHT', 'sheet', 'лист'),
        ('SM3', 'standard cubic metre', 'стандартен кубичен метър'),
        ('ST', 'sheet', 'лист'),
        ('STK', 'stick', 'пръчка'),
        ('TRL', 'trillion', 'трилион'),
        ('VLT', 'volt', 'волт'),
        ('WTT', 'watt', 'ват'),
        ('ZZ', 'mutually defined', 'взаимно определено')
      ON CONFLICT (code) DO NOTHING
    SQL
  end

  def rollback
    execute "DROP TABLE IF EXISTS saft_regions"
    execute "DROP TABLE IF EXISTS saft_iban_formats"
    # Don't drop extended unit of measure data, just the new entries
    execute <<-SQL
      DELETE FROM saft_units_of_measure
      WHERE code IN ('10', '11', '27', '58', '2N', 'A11', 'A39', 'A43', 'A86', 'B10', 'B49', 'C36', 'D63', 'E14', 'E27', 'E49', 'FB', 'H87', 'HTZ', 'INH', 'JOU', 'KGS', 'KHZ', 'KMK', 'KPA', 'KWN', 'LEF', 'MBR', 'MGM', 'MHZ', 'MIK', 'MPA', 'NAR', 'NPR', 'NRL', 'OHM', 'P1', 'PA', 'PAL', 'PFL', 'RPM', 'SEC', 'SHT', 'SM3', 'ST', 'STK', 'TRL', 'VLT', 'WTT', 'ZZ')
    SQL
  end
end
