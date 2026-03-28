# VatJournalEntry - Запис в ДДС дневник (покупки или продажби)
# Used for generating POKUPKI.TXT and PRODAJBI.TXT files for NAP
class VatJournalEntry < BaseModel
  table do
    # Common fields for both purchases and sales
    column row_number : Int32                          # Пореден номер
    column document_type : String                       # Вид документ (01-95)
    column document_number : String                     # Номер на документа (20 символа)
    column document_date : Time                         # Дата на документа
    column counterpart_vat : String?                    # ИН на контрагента (15 символа)
    column counterpart_name : String?                   # Име на контрагента (50 символа)
    column goods_description : String?                  # Вид на стоката/услугата (30 символа)
    column delivery_code : String?                      # Колона 8а: 01,02,03,07,08,41,43,46,48,51,53,54,58
    column entry_type : String                          # purchase / sales

    # Purchase columns (POKUPKI.TXT columns 03-30 to 03-44)
    column base_no_credit : Float64 = 0.0              # 03-30: ДО без право на данъчен кредит
    column base_full_credit : Float64 = 0.0            # 03-31: ДО с право на пълен данъчен кредит
    column vat_full_credit : Float64 = 0.0             # 03-32: ДДС с право на пълен данъчен кредит
    column base_partial_credit : Float64 = 0.0         # 03-33: ДО с право на частичен данъчен кредит
    column vat_partial_credit : Float64 = 0.0          # 03-34: ДДС с право на частичен данъчен кредит
    column annual_adjustment_base : Float64 = 0.0      # 03-35: Годишна корекция ДО
    column annual_adjustment_vat : Float64 = 0.0       # 03-36: Годишна корекция ДДС
    column base_20 : Float64 = 0.0                     # 03-40: ДО при ставка 20%
    column vat_20 : Float64 = 0.0                      # 03-41: ДДС при ставка 20%
    column vop_base : Float64 = 0.0                    # 03-42: ДО при ВОП
    column vop_vat : Float64 = 0.0                     # 03-43: ДДС при ВОП
    column base_9 : Float64 = 0.0                      # 03-46: ДО при 9% ставка
    column vat_9 : Float64 = 0.0                       # 03-47: ДДС при 9% ставка

    # Sales columns (PRODAJBI.TXT columns 02-10 to 02-26)
    column total_base : Float64 = 0.0                  # 02-10: Общ размер на данъчната основа
    column total_vat : Float64 = 0.0                   # 02-11: Всичко начислен ДДС
    column sales_base_20 : Float64 = 0.0               # 02-12: ДО облагаеми със ставка 20%
    column sales_vat_20 : Float64 = 0.0                # 02-13: ДДС 20%
    column sales_base_vop : Float64 = 0.0              # 02-14: ДО при ВОП и чл.82
    column sales_vat_vop : Float64 = 0.0               # 02-15: ДДС при ВОП и чл.82
    column sales_base_9 : Float64 = 0.0                # 02-16: ДО облагаеми със ставка 9%
    column sales_vat_9 : Float64 = 0.0                 # 02-17: ДДС 9%
    column sales_base_0_chapter3 : Float64 = 0.0       # 02-18: ДО облагаеми със ставка 0% по глава 3
    column sales_base_vod : Float64 = 0.0              # 02-19: ДО на ВОД (вътреобщностни доставки)
    column sales_base_0_articles : Float64 = 0.0       # 02-20: ДО по чл.140,146,173
    column sales_base_services_21 : Float64 = 0.0      # 02-21: Услуги по чл.21 ал.2
    column sales_base_69_2 : Float64 = 0.0             # 02-22: ДО по чл.69 ал.2
    column sales_base_69_2_eu : Float64 = 0.0          # 02-23: ДО по чл.69 ал.2 към ЕС
    column sales_base_exempt : Float64 = 0.0           # 02-24: Освободени доставки
    column sales_vat_personal : Float64 = 0.0          # 02-25: ДДС за лични нужди
    column sales_base_vop_9 : Float64 = 0.0            # 02-26: ДО при ВОП със ставка 9%

    # Additional metadata
    column source_journal_entry_id : Int64?            # Link to original journal entry if imported
    column notes : String?                             # Additional notes

    belongs_to company : Company
    belongs_to vat_return : VatReturn
  end

  # Document type codes for purchases (Покупки)
  PURCHASE_DOC_TYPES = {
    "01" => "Фактура",
    "02" => "Дебитно известие",
    "03" => "Кредитно известие",
    "05" => "Регистър стоки до поискване",
    "07" => "Митническа декларация",
    "09" => "Протокол",
    "11" => "Фактура - касова отчетност",
    "12" => "ДИ - касова отчетност",
    "13" => "КИ - касова отчетност",
    "23" => "КИ по чл.126б",
    "91" => "Протокол чл.151в ал.3",
    "92" => "Протокол чл.151г ал.8",
    "93" => "Протокол чл.151в ал.7 (без режим)",
    "94" => "Протокол чл.151в ал.7 (с режим)",
  }

  # Document type codes for sales (Продажби)
  SALES_DOC_TYPES = {
    "01" => "Фактура",
    "02" => "Дебитно известие",
    "03" => "Кредитно известие",
    "04" => "Регистър стоки до поискване",
    "09" => "Протокол",
    "11" => "Фактура - касова отчетност",
    "12" => "ДИ - касова отчетност",
    "13" => "КИ - касова отчетност",
    "29" => "КИ по чл.126б",
    "50" => "Известие за продажба",
    "81" => "Отчет за извършени продажби",
    "82" => "Отчет продажби чл.119",
    "83" => "Отчет за безвъзмездни доставки",
    "84" => "Отчет за корекции на ДДС",
    "85" => "Отчет доставки чл.163а",
    "95" => "Протокол по чл.117 ал.4",
  }

  # Delivery codes for column 8а
  DELIVERY_CODES = {
    "01" => "Доставка по част I Приложение 2",
    "02" => "Доставка по част II Приложение 2",
    "03" => "Внос по Приложение 3",
    "07" => "Доставка по чл.163а",
    "08" => "Доставка по чл.163б",
    "41" => "Изпращане стоки режим складиране",
    "43" => "Замяна на лицето чл.15а ал.4",
    "46" => "Корекция грешен ИН",
    "48" => "Връщане на стоки чл.15а ал.5",
    "51" => "Пристигане стоки режим складиране",
    "53" => "Замяна без прекратяване",
    "54" => "Брак/липса/унищожаване",
    "58" => "Прекратяване на договора",
  }

  # Document codes that should be excluded from the declaration (СД)
  # These are reported but not summed in the declaration
  EXCLUDED_FROM_DECLARATION = ["04", "05", "11", "12", "13", "94"]

  def purchase?
    entry_type == "purchase"
  end

  def sales?
    entry_type == "sales"
  end

  def excluded_from_declaration?
    EXCLUDED_FROM_DECLARATION.includes?(document_type)
  end

  # Format date for NAP export (dd/mm/yyyy)
  def formatted_date
    document_date.to_s("%d/%m/%Y")
  end

  # Get document type name
  def document_type_name
    if purchase?
      PURCHASE_DOC_TYPES[document_type]? || document_type
    else
      SALES_DOC_TYPES[document_type]? || document_type
    end
  end

  # Get delivery code name
  def delivery_code_name
    return nil unless delivery_code
    DELIVERY_CODES[delivery_code]?
  end
end
