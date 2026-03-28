# List VAT returns for a company
class Api::VatReturns::Index < ApiAction
  include Api::Auth::SkipRequireAuthToken

  get "/api/companies/:company_id/vat_returns" do
    returns = VatReturnQuery.new
      .company_id(company_id)
      .created_at.desc_order

    json({
      success: true,
      data:    returns.map { |r| vat_return_json(r) },
    })
  end

  private def vat_return_json(vr : VatReturn)
    result = vr.result_json ? JSON.parse(vr.result_json.not_nil!) : JSON::Any.new({} of String => JSON::Any)
    vat_due = result["vat_due"]?.try(&.as_f?) || 0.0

    {
      id:                   vr.id,
      period_year:          vr.period_year,
      period_month:         vr.period_month,
      status:               vr.status,
      vat_due:              vat_due,
      result_json:          vr.result_json,
      # Metadata
      representative_name:  vr.representative_name,
      representative_egn:   vr.representative_egn,
      sales_count:          vr.sales_count,
      purchase_count:       vr.purchase_count,
      # Section A - Output VAT
      cell_01_01:           vr.cell_01_01,
      cell_01_02:           vr.cell_01_02,
      cell_01_03:           vr.cell_01_03,
      cell_01_04:           vr.cell_01_04,
      cell_01_05:           vr.cell_01_05,
      cell_01_06:           vr.cell_01_06,
      cell_01_07:           vr.cell_01_07,
      cell_01_11:           vr.cell_01_11,
      cell_01_12:           vr.cell_01_12,
      cell_01_13:           vr.cell_01_13,
      cell_01_14:           vr.cell_01_14,
      cell_01_15:           vr.cell_01_15,
      cell_01_16:           vr.cell_01_16,
      cell_01_17:           vr.cell_01_17,
      cell_01_18:           vr.cell_01_18,
      # Section B - Input VAT
      cell_01_20:           vr.cell_01_20,
      cell_01_21:           vr.cell_01_21,
      cell_01_22:           vr.cell_01_22,
      cell_01_23:           vr.cell_01_23,
      cell_01_24:           vr.cell_01_24,
      cell_01_25:           vr.cell_01_25,
      cell_01_26:           vr.cell_01_26,
      cell_01_30:           vr.cell_01_30,
      cell_01_31:           vr.cell_01_31,
      cell_01_32:           vr.cell_01_32,
      cell_01_33:           vr.cell_01_33,
      # Section C - Result
      cell_01_40:           vr.cell_01_40,
      cell_01_41:           vr.cell_01_41,
      cell_01_42:           vr.cell_01_42,
      cell_01_43:           vr.cell_01_43,
      cell_01_50:           vr.cell_01_50,
      cell_01_51:           vr.cell_01_51,
      cell_01_52:           vr.cell_01_52,
      cell_01_60:           vr.cell_01_60,
    }
  end
end
