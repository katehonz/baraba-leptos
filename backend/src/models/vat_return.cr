# VAT Return - ДДС декларация
class VatReturn < BaseModel
  table do
    column period_year : Int32
    column period_month : Int32
    column status : String = "draft"  # draft, calculated, submitted
    column result_json : String?              # Calculated amounts as JSON

    # Declaration metadata
    column representative_name : String?      # 00-04: Лице подаващо данните
    column representative_egn : String?       # 00-05: ЕГН на лицето
    column sales_count : Int32 = 0            # 00-08: Брой записи в дневник продажби
    column purchase_count : Int32 = 0         # 00-09: Брой записи в дневник покупки

    # SECTION A - OUTPUT VAT (Sales/Начислен ДДС)
    column cell_01_01 : Float64 = 0.0         # Данъчна основа на облагаеми доставки със ставка 20%
    column cell_01_02 : Float64 = 0.0         # Начислен ДДС 20%
    column cell_01_03 : Float64 = 0.0         # ДО при ВОП и чл.82 ал.2-5
    column cell_01_04 : Float64 = 0.0         # Начислен ДДС при ВОП
    column cell_01_05 : Float64 = 0.0         # ДО облагаеми доставки със ставка 9%
    column cell_01_06 : Float64 = 0.0         # Начислен ДДС 9%
    column cell_01_07 : Float64 = 0.0         # ДО облагаеми със ставка 0% по глава 3
    column cell_01_11 : Float64 = 0.0         # ДО на ВОД (вътреобщностни доставки)
    column cell_01_12 : Float64 = 0.0         # ДО доставки по чл.140, 146, 173
    column cell_01_13 : Float64 = 0.0         # ДО на услуги по чл.21 ал.2
    column cell_01_14 : Float64 = 0.0         # ДО по чл.69 ал.2 - доставки като посредник
    column cell_01_15 : Float64 = 0.0         # ДО по чл.69 ал.2 в друга ДЧ
    column cell_01_16 : Float64 = 0.0         # ДО на освободени доставки по глава 4
    column cell_01_17 : Float64 = 0.0         # ДДС за лични нужди по чл.6 ал.3
    column cell_01_18 : Float64 = 0.0         # ДО при ВОП със ставка 9%

    # SECTION B - INPUT TAX CREDIT (Purchases/Данъчен кредит)
    column cell_01_20 : Float64 = 0.0         # ДО на получени доставки без право на ДК
    column cell_01_21 : Float64 = 0.0         # ДО на получени доставки с право на пълен ДК
    column cell_01_22 : Float64 = 0.0         # ДДС с право на пълен данъчен кредит
    column cell_01_23 : Float64 = 0.0         # ДО на получени доставки с право на частичен ДК
    column cell_01_24 : Float64 = 0.0         # ДДС с право на частичен данъчен кредит
    column cell_01_25 : Float64 = 0.0         # Годишна корекция - ДО
    column cell_01_26 : Float64 = 0.0         # Годишна корекция - ДДС
    column cell_01_30 : Float64 = 0.0         # ДО по чл.163а ал.2 (златни изделия)
    column cell_01_31 : Float64 = 0.0         # ДДС по чл.163а ал.2
    column cell_01_32 : Float64 = 0.0         # ДО при ВОП
    column cell_01_33 : Float64 = 0.0         # ДДС при ВОП

    # SECTION V - RESULT (Резултат)
    column cell_01_40 : Float64 = 0.0         # Общо начислен ДДС
    column cell_01_41 : Float64 = 0.0         # Данък за внасяне
    column cell_01_42 : Float64 = 0.0         # ДДС за възстановяване
    column cell_01_43 : Float64 = 0.0         # Ефективно внесен данък
    column cell_01_50 : Float64 = 0.0         # ДДС за внасяне от предходен период
    column cell_01_51 : Float64 = 0.0         # Данък за внасяне (01-41 + 01-50)
    column cell_01_52 : Float64 = 0.0         # ДДС за възстановяване (01-42 - 01-50)
    column cell_01_60 : Float64 = 0.0         # Данъчен кредит за приспадане

    belongs_to company : Company
    has_many vat_journal_entries : VatJournalEntry
  end

  # Get period as string (YYYY-MM)
  def period_string
    "#{period_year}-#{period_month.to_s.rjust(2, '0')}"
  end

  # Get period for display (MM/YYYY)
  def period_display
    "#{period_month.to_s.rjust(2, '0')}/#{period_year}"
  end

  # Calculate total output VAT (начислен ДДС)
  def total_output_vat
    cell_01_02 + cell_01_04 + cell_01_06 + cell_01_17
  end

  # Calculate total input VAT credit (данъчен кредит)
  def total_input_vat
    cell_01_22 + cell_01_24 + cell_01_26 + cell_01_31 + cell_01_33
  end

  # Calculate VAT due (positive) or refund (negative)
  def vat_result
    total_output_vat - total_input_vat
  end
end
