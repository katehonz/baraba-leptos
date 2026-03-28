# Migration for VatJournalEntry table
# Specialized table for VAT journals (POKUPKI.TXT and PRODAJBI.TXT)
class CreateVatJournalEntries::V20250126000047 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(VatJournalEntry) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add_belongs_to vat_return : VatReturn, on_delete: :cascade

      # Common fields for both purchases and sales
      add row_number : Int32                          # Пореден номер
      add branch_code : String, default: "0001"       # Клон (4 цифри)
      add document_type : String                       # Вид документ (01-95)
      add document_number : String                     # Номер на документа (20 символа)
      add document_date : Time                         # Дата на документа
      add counterpart_vat : String?                    # ИН на контрагента (15 символа)
      add counterpart_name : String?                   # Име на контрагента (50 символа)
      add goods_description : String?                  # Вид на стоката/услугата (30 символа)
      add delivery_code : String?                      # Колона 8а: 01,02,03,07,08,41,43,46,48,51,53,54,58
      add entry_type : String                          # purchase / sales

      # Purchase columns (POKUPKI.TXT columns 03-30 to 03-44)
      add base_no_credit : Float64, default: 0.0       # 03-30: ДО без право на данъчен кредит
      add base_full_credit : Float64, default: 0.0     # 03-31: ДО с право на пълен данъчен кредит
      add vat_full_credit : Float64, default: 0.0      # 03-32: ДДС с право на пълен данъчен кредит
      add base_partial_credit : Float64, default: 0.0  # 03-33: ДО с право на частичен данъчен кредит
      add vat_partial_credit : Float64, default: 0.0   # 03-34: ДДС с право на частичен данъчен кредит
      add annual_adjustment_base : Float64, default: 0.0  # 03-35: Годишна корекция ДО
      add annual_adjustment_vat : Float64, default: 0.0   # 03-36: Годишна корекция ДДС
      add base_20 : Float64, default: 0.0              # 03-40: ДО при ставка 20%
      add vat_20 : Float64, default: 0.0               # 03-41: ДДС при ставка 20%
      add vop_base : Float64, default: 0.0             # 03-42: ДО при ВОП
      add vop_vat : Float64, default: 0.0              # 03-43: ДДС при ВОП
      add base_9 : Float64, default: 0.0               # For 9% rate purchases
      add vat_9 : Float64, default: 0.0                # VAT at 9%

      # Sales columns (PRODAJBI.TXT columns 02-10 to 02-26)
      add total_base : Float64, default: 0.0           # 02-10: Общ размер на данъчната основа
      add total_vat : Float64, default: 0.0            # 02-11: Всичко начислен ДДС
      add sales_base_20 : Float64, default: 0.0        # 02-12: ДО облагаеми със ставка 20%
      add sales_vat_20 : Float64, default: 0.0         # 02-13: ДДС 20%
      add sales_base_vop : Float64, default: 0.0       # 02-14: ДО при ВОП и чл.82
      add sales_vat_vop : Float64, default: 0.0        # 02-15: ДДС при ВОП и чл.82
      add sales_base_9 : Float64, default: 0.0         # 02-16: ДО облагаеми със ставка 9%
      add sales_vat_9 : Float64, default: 0.0          # 02-17: ДДС 9%
      add sales_base_0_chapter3 : Float64, default: 0.0  # 02-18: ДО облагаеми със ставка 0% по глава 3
      add sales_base_vod : Float64, default: 0.0       # 02-19: ДО на ВОД (вътреобщностни доставки)
      add sales_base_0_articles : Float64, default: 0.0  # 02-20: ДО по чл.140,146,173
      add sales_base_services_21 : Float64, default: 0.0 # 02-21: Услуги по чл.21 ал.2
      add sales_base_69_2 : Float64, default: 0.0      # 02-22: ДО по чл.69 ал.2
      add sales_base_69_2_eu : Float64, default: 0.0   # 02-23: ДО по чл.69 ал.2 към ЕС
      add sales_base_exempt : Float64, default: 0.0    # 02-24: Освободени доставки
      add sales_vat_personal : Float64, default: 0.0   # 02-25: ДДС за лични нужди
      add sales_base_vop_9 : Float64, default: 0.0     # 02-26: ДО при ВОП със ставка 9%

      # Additional metadata
      add source_journal_entry_id : Int64?             # Link to original journal entry if imported
      add notes : String?                              # Additional notes
    end

    # Create indexes
    create_index :vat_journal_entries, [:vat_return_id, :entry_type]
    create_index :vat_journal_entries, [:company_id, :document_number]
    create_index :vat_journal_entries, [:vat_return_id, :row_number]
  end

  def rollback
    drop table_for(VatJournalEntry)
  end
end
