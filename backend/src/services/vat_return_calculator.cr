require "big"

# Aggregates VatJournalEntry rows into declaration cells and result_json.
class VatReturnCalculator
  EXCLUDED_CODES = ["04", "05", "11", "12", "13", "94"]

  def self.calculate!(vat_return : VatReturn, company : Company?) : VatReturn
    vrid = vat_return.id

    rep_name = ""
    rep_egn = ""
    if company
      if company.representative_type == "AUTHORIZED_PERSON"
        rep_name = company.authorized_person_name || ""
        rep_egn = company.authorized_person_egn || ""
      else
        rep_name = company.manager_name || ""
        rep_egn = company.manager_egn || ""
      end
    end

    purchases = VatJournalEntryQuery.new
      .vat_return_id(vrid)
      .entry_type("purchase")

    sales = VatJournalEntryQuery.new
      .vat_return_id(vrid)
      .entry_type("sales")

    cell_01_01 = BigDecimal.new(0)
    cell_01_02 = BigDecimal.new(0)
    cell_01_03 = BigDecimal.new(0)
    cell_01_04 = BigDecimal.new(0)
    cell_01_05 = BigDecimal.new(0)
    cell_01_06 = BigDecimal.new(0)
    cell_01_07 = BigDecimal.new(0)
    cell_01_11 = BigDecimal.new(0)
    cell_01_12 = BigDecimal.new(0)
    cell_01_13 = BigDecimal.new(0)
    cell_01_14 = BigDecimal.new(0)
    cell_01_15 = BigDecimal.new(0)
    cell_01_16 = BigDecimal.new(0)
    cell_01_17 = BigDecimal.new(0)
    cell_01_18 = BigDecimal.new(0)

    cell_01_20 = BigDecimal.new(0)
    cell_01_21 = BigDecimal.new(0)
    cell_01_22 = BigDecimal.new(0)
    cell_01_23 = BigDecimal.new(0)
    cell_01_24 = BigDecimal.new(0)
    cell_01_25 = BigDecimal.new(0)
    cell_01_26 = BigDecimal.new(0)
    cell_01_30 = BigDecimal.new(0)
    cell_01_31 = BigDecimal.new(0)
    cell_01_32 = BigDecimal.new(0)
    cell_01_33 = BigDecimal.new(0)

    purchase_count = 0
    sales_count = 0

    purchases.each do |entry|
      purchase_count += 1
      next if EXCLUDED_CODES.includes?(entry.document_type)

      cell_01_20 += BigDecimal.new(entry.base_no_credit)
      cell_01_21 += BigDecimal.new(entry.base_full_credit)
      cell_01_22 += BigDecimal.new(entry.vat_full_credit)
      cell_01_23 += BigDecimal.new(entry.base_partial_credit)
      cell_01_24 += BigDecimal.new(entry.vat_partial_credit)
      cell_01_25 += BigDecimal.new(entry.annual_adjustment_base)
      cell_01_26 += BigDecimal.new(entry.annual_adjustment_vat)
      cell_01_30 += BigDecimal.new(entry.base_20)
      cell_01_31 += BigDecimal.new(entry.vat_20)
      cell_01_32 += BigDecimal.new(entry.vop_base)
      cell_01_33 += BigDecimal.new(entry.vop_vat)
    end

    sales.each do |entry|
      sales_count += 1
      next if EXCLUDED_CODES.includes?(entry.document_type)

      cell_01_01 += BigDecimal.new(entry.sales_base_20)
      cell_01_02 += BigDecimal.new(entry.sales_vat_20)
      cell_01_03 += BigDecimal.new(entry.sales_base_vop)
      cell_01_04 += BigDecimal.new(entry.sales_vat_vop)
      cell_01_05 += BigDecimal.new(entry.sales_base_9)
      cell_01_06 += BigDecimal.new(entry.sales_vat_9)
      cell_01_07 += BigDecimal.new(entry.sales_base_0_chapter3)
      cell_01_11 += BigDecimal.new(entry.sales_base_vod)
      cell_01_12 += BigDecimal.new(entry.sales_base_0_articles)
      cell_01_13 += BigDecimal.new(entry.sales_base_services_21)
      cell_01_14 += BigDecimal.new(entry.sales_base_69_2)
      cell_01_15 += BigDecimal.new(entry.sales_base_69_2_eu)
      cell_01_16 += BigDecimal.new(entry.sales_base_exempt)
      cell_01_17 += BigDecimal.new(entry.sales_vat_personal)
      cell_01_18 += BigDecimal.new(entry.sales_base_vop_9)
    end

    cell_01_40 = cell_01_02 + cell_01_04 + cell_01_06 + cell_01_17
    total_input_vat = cell_01_22 + cell_01_24 + cell_01_26 + cell_01_31 + cell_01_33
    vat_result = cell_01_40 - total_input_vat
    cell_01_41 = vat_result > 0 ? vat_result : BigDecimal.new(0)
    cell_01_42 = vat_result < 0 ? vat_result.abs : BigDecimal.new(0)
    cell_01_51 = cell_01_41
    cell_01_52 = cell_01_42

    # Dashboard "Покупки 20%" is full-credit base/VAT (01-21 / 01-22), NOT чл.163а (01-30).
    result = {
      purchase_base_20:   cell_01_21.to_f,
      purchase_vat_20:    cell_01_22.to_f,
      purchase_base_9:    cell_01_23.to_f,
      purchase_vat_9:     cell_01_24.to_f,
      purchase_base_0:    cell_01_20.to_f,
      purchase_intra_eu:  cell_01_32.to_f,
      total_purchase_vat: total_input_vat.to_f,
      sales_base_20:      cell_01_01.to_f,
      sales_vat_20:       cell_01_02.to_f,
      sales_base_9:       cell_01_05.to_f,
      sales_vat_9:        cell_01_06.to_f,
      sales_base_0:       cell_01_07.to_f,
      sales_intra_eu:     cell_01_11.to_f,
      sales_exempt:       cell_01_16.to_f,
      total_sales_vat:    cell_01_40.to_f,
      vat_due:            vat_result.to_f,
    }

    SaveVatReturn.update!(vat_return,
      status: "calculated",
      result_json: result.to_json,
      sales_count: sales_count,
      purchase_count: purchase_count,
      representative_name: rep_name,
      representative_egn: rep_egn,
      cell_01_01: cell_01_01.to_f,
      cell_01_02: cell_01_02.to_f,
      cell_01_03: cell_01_03.to_f,
      cell_01_04: cell_01_04.to_f,
      cell_01_05: cell_01_05.to_f,
      cell_01_06: cell_01_06.to_f,
      cell_01_07: cell_01_07.to_f,
      cell_01_11: cell_01_11.to_f,
      cell_01_12: cell_01_12.to_f,
      cell_01_13: cell_01_13.to_f,
      cell_01_14: cell_01_14.to_f,
      cell_01_15: cell_01_15.to_f,
      cell_01_16: cell_01_16.to_f,
      cell_01_17: cell_01_17.to_f,
      cell_01_18: cell_01_18.to_f,
      cell_01_20: cell_01_20.to_f,
      cell_01_21: cell_01_21.to_f,
      cell_01_22: cell_01_22.to_f,
      cell_01_23: cell_01_23.to_f,
      cell_01_24: cell_01_24.to_f,
      cell_01_25: cell_01_25.to_f,
      cell_01_26: cell_01_26.to_f,
      cell_01_30: cell_01_30.to_f,
      cell_01_31: cell_01_31.to_f,
      cell_01_32: cell_01_32.to_f,
      cell_01_33: cell_01_33.to_f,
      cell_01_40: cell_01_40.to_f,
      cell_01_41: cell_01_41.to_f,
      cell_01_42: cell_01_42.to_f,
      cell_01_51: cell_01_51.to_f,
      cell_01_52: cell_01_52.to_f
    )
  end
end
