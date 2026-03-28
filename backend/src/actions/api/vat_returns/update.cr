# Update VAT return declaration fields
class Api::VatReturns::Update < ApiAction
  include Api::Auth::SkipRequireAuthToken

  put "/api/companies/:company_id/vat_returns/:vat_return_id" do
    vat_return = VatReturnQuery.new
      .company_id(company_id)
      .id(vat_return_id)
      .first?

    unless vat_return
      return json({success: false, error: "VAT return not found"})
    end

    # Parse JSON body
    body_str = request.body.try(&.gets_to_end) || "{}"
    json_body = JSON.parse(body_str)

    # Update fields
    SaveVatReturn.update!(vat_return,
      representative_name: json_body["representative_name"]?.try(&.as_s?),
      representative_egn: json_body["representative_egn"]?.try(&.as_s?),
      # Section A - Output VAT
      cell_01_01: json_body["cell_01_01"]?.try(&.as_f?) || vat_return.cell_01_01,
      cell_01_02: json_body["cell_01_02"]?.try(&.as_f?) || vat_return.cell_01_02,
      cell_01_03: json_body["cell_01_03"]?.try(&.as_f?) || vat_return.cell_01_03,
      cell_01_04: json_body["cell_01_04"]?.try(&.as_f?) || vat_return.cell_01_04,
      cell_01_05: json_body["cell_01_05"]?.try(&.as_f?) || vat_return.cell_01_05,
      cell_01_06: json_body["cell_01_06"]?.try(&.as_f?) || vat_return.cell_01_06,
      cell_01_07: json_body["cell_01_07"]?.try(&.as_f?) || vat_return.cell_01_07,
      cell_01_11: json_body["cell_01_11"]?.try(&.as_f?) || vat_return.cell_01_11,
      cell_01_12: json_body["cell_01_12"]?.try(&.as_f?) || vat_return.cell_01_12,
      cell_01_13: json_body["cell_01_13"]?.try(&.as_f?) || vat_return.cell_01_13,
      cell_01_14: json_body["cell_01_14"]?.try(&.as_f?) || vat_return.cell_01_14,
      cell_01_15: json_body["cell_01_15"]?.try(&.as_f?) || vat_return.cell_01_15,
      cell_01_16: json_body["cell_01_16"]?.try(&.as_f?) || vat_return.cell_01_16,
      cell_01_17: json_body["cell_01_17"]?.try(&.as_f?) || vat_return.cell_01_17,
      cell_01_18: json_body["cell_01_18"]?.try(&.as_f?) || vat_return.cell_01_18,
      # Section B - Input VAT
      cell_01_20: json_body["cell_01_20"]?.try(&.as_f?) || vat_return.cell_01_20,
      cell_01_21: json_body["cell_01_21"]?.try(&.as_f?) || vat_return.cell_01_21,
      cell_01_22: json_body["cell_01_22"]?.try(&.as_f?) || vat_return.cell_01_22,
      cell_01_23: json_body["cell_01_23"]?.try(&.as_f?) || vat_return.cell_01_23,
      cell_01_24: json_body["cell_01_24"]?.try(&.as_f?) || vat_return.cell_01_24,
      cell_01_25: json_body["cell_01_25"]?.try(&.as_f?) || vat_return.cell_01_25,
      cell_01_26: json_body["cell_01_26"]?.try(&.as_f?) || vat_return.cell_01_26,
      cell_01_30: json_body["cell_01_30"]?.try(&.as_f?) || vat_return.cell_01_30,
      cell_01_31: json_body["cell_01_31"]?.try(&.as_f?) || vat_return.cell_01_31,
      cell_01_32: json_body["cell_01_32"]?.try(&.as_f?) || vat_return.cell_01_32,
      cell_01_33: json_body["cell_01_33"]?.try(&.as_f?) || vat_return.cell_01_33,
      # Section C - Result
      cell_01_40: json_body["cell_01_40"]?.try(&.as_f?) || vat_return.cell_01_40,
      cell_01_41: json_body["cell_01_41"]?.try(&.as_f?) || vat_return.cell_01_41,
      cell_01_42: json_body["cell_01_42"]?.try(&.as_f?) || vat_return.cell_01_42,
      cell_01_43: json_body["cell_01_43"]?.try(&.as_f?) || vat_return.cell_01_43,
      cell_01_50: json_body["cell_01_50"]?.try(&.as_f?) || vat_return.cell_01_50,
      cell_01_51: json_body["cell_01_51"]?.try(&.as_f?) || vat_return.cell_01_51,
      cell_01_52: json_body["cell_01_52"]?.try(&.as_f?) || vat_return.cell_01_52,
      cell_01_60: json_body["cell_01_60"]?.try(&.as_f?) || vat_return.cell_01_60
    )

    # Reload and return
    updated = VatReturnQuery.new.id(vat_return.id).first
    json({
      success: true,
      message: "Декларацията е запазена",
      data:    {id: updated.id},
    })
  rescue ex
    json({success: false, error: ex.message})
  end
end
