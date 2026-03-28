# Migration to extend VatReturn with additional fields for DEKLAR.TXT
class ExtendVatReturns::V20250126000048 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(VatReturn) do
      # Declaration metadata
      add representative_name : String?              # 00-04: Лице подаващо данните
      add representative_egn : String?               # 00-05: ЕГН на лицето
      add sales_count : Int32, default: 0            # 00-08: Брой записи в дневник продажби
      add purchase_count : Int32, default: 0         # 00-09: Брой записи в дневник покупки

      # SECTION A - OUTPUT VAT (Sales/Начислен ДДС)
      # Cell 01-XX corresponds to declaration box number
      add cell_01_01 : Float64, default: 0.0         # Данъчна основа на облагаеми доставки със ставка 20%
      add cell_01_02 : Float64, default: 0.0         # Начислен ДДС 20%
      add cell_01_03 : Float64, default: 0.0         # ДО при ВОП и чл.82 ал.2-5
      add cell_01_04 : Float64, default: 0.0         # Начислен ДДС при ВОП
      add cell_01_05 : Float64, default: 0.0         # ДО облагаеми доставки със ставка 9%
      add cell_01_06 : Float64, default: 0.0         # Начислен ДДС 9%
      add cell_01_07 : Float64, default: 0.0         # ДО облагаеми със ставка 0% по глава 3
      add cell_01_11 : Float64, default: 0.0         # ДО на ВОД (вътреобщностни доставки)
      add cell_01_12 : Float64, default: 0.0         # ДО доставки по чл.140, 146, 173
      add cell_01_13 : Float64, default: 0.0         # ДО на услуги по чл.21 ал.2
      add cell_01_14 : Float64, default: 0.0         # ДО по чл.69 ал.2 - доставки като посредник
      add cell_01_15 : Float64, default: 0.0         # ДО по чл.69 ал.2 в друга ДЧ
      add cell_01_16 : Float64, default: 0.0         # ДО на освободени доставки по глава 4
      add cell_01_17 : Float64, default: 0.0         # ДДС за лични нужди по чл.6 ал.3
      add cell_01_18 : Float64, default: 0.0         # ДО при ВОП със ставка 9%

      # SECTION B - INPUT TAX CREDIT (Purchases/Данъчен кредит)
      add cell_01_20 : Float64, default: 0.0         # ДО на получени доставки без право на ДК
      add cell_01_21 : Float64, default: 0.0         # ДО на получени доставки с право на пълен ДК
      add cell_01_22 : Float64, default: 0.0         # ДДС с право на пълен данъчен кредит
      add cell_01_23 : Float64, default: 0.0         # ДО на получени доставки с право на частичен ДК
      add cell_01_24 : Float64, default: 0.0         # ДДС с право на частичен данъчен кредит
      add cell_01_25 : Float64, default: 0.0         # Годишна корекция - ДО
      add cell_01_26 : Float64, default: 0.0         # Годишна корекция - ДДС
      add cell_01_30 : Float64, default: 0.0         # ДО по чл.163а ал.2 (златни изделия)
      add cell_01_31 : Float64, default: 0.0         # ДДС по чл.163а ал.2
      add cell_01_32 : Float64, default: 0.0         # ДО при ВОП
      add cell_01_33 : Float64, default: 0.0         # ДДС при ВОП

      # SECTION V - RESULT (Резултат)
      add cell_01_40 : Float64, default: 0.0         # Общо начислен ДДС (сума от 01-02,01-04,01-06,01-17)
      add cell_01_41 : Float64, default: 0.0         # Данък за внасяне
      add cell_01_42 : Float64, default: 0.0         # ДДС за възстановяване
      add cell_01_43 : Float64, default: 0.0         # Ефективно внесен данък
      add cell_01_50 : Float64, default: 0.0         # ДДС за внасяне от предходен период
      add cell_01_51 : Float64, default: 0.0         # Данък за внасяне (01-41 + 01-50)
      add cell_01_52 : Float64, default: 0.0         # ДДС за възстановяване (01-42 - 01-50)
      add cell_01_60 : Float64, default: 0.0         # Данъчен кредит за приспадане
    end
  end

  def rollback
    alter table_for(VatReturn) do
      remove :representative_name
      remove :representative_egn
      remove :sales_count
      remove :purchase_count
      remove :cell_01_01
      remove :cell_01_02
      remove :cell_01_03
      remove :cell_01_04
      remove :cell_01_05
      remove :cell_01_06
      remove :cell_01_07
      remove :cell_01_11
      remove :cell_01_12
      remove :cell_01_13
      remove :cell_01_14
      remove :cell_01_15
      remove :cell_01_16
      remove :cell_01_17
      remove :cell_01_18
      remove :cell_01_20
      remove :cell_01_21
      remove :cell_01_22
      remove :cell_01_23
      remove :cell_01_24
      remove :cell_01_25
      remove :cell_01_26
      remove :cell_01_30
      remove :cell_01_31
      remove :cell_01_32
      remove :cell_01_33
      remove :cell_01_40
      remove :cell_01_41
      remove :cell_01_42
      remove :cell_01_43
      remove :cell_01_50
      remove :cell_01_51
      remove :cell_01_52
      remove :cell_01_60
    end
  end
end
