class SaveVatReturn < VatReturn::SaveOperation
  permit_columns(
    period_year,
    period_month,
    status,
    result_json,
    company_id,
    # Declaration metadata
    representative_name,
    representative_egn,
    sales_count,
    purchase_count,
    # Section A - Output VAT
    cell_01_01,
    cell_01_02,
    cell_01_03,
    cell_01_04,
    cell_01_05,
    cell_01_06,
    cell_01_07,
    cell_01_11,
    cell_01_12,
    cell_01_13,
    cell_01_14,
    cell_01_15,
    cell_01_16,
    cell_01_17,
    cell_01_18,
    # Section B - Input VAT Credit
    cell_01_20,
    cell_01_21,
    cell_01_22,
    cell_01_23,
    cell_01_24,
    cell_01_25,
    cell_01_26,
    cell_01_30,
    cell_01_31,
    cell_01_32,
    cell_01_33,
    # Section V - Result
    cell_01_40,
    cell_01_41,
    cell_01_42,
    cell_01_43,
    cell_01_50,
    cell_01_51,
    cell_01_52,
    cell_01_60
  )

  before_save do
    validate_required period_year, period_month
    validate_inclusion_of status, in: ["draft", "calculated", "submitted"]
    validate_inclusion_of period_month, in: (1..12).to_a
  end
end
