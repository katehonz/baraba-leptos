class Api::Nomenclatures::VatExemptions < ApiAction
  get "/api/nomenclatures/vat_exemptions" do
    rows = VatExemptionReasonQuery.new.is_active(true).sort_order.asc_order
    json({
      success: true,
      data:    rows.map { |r|
        {
          id:                 r.id,
          reason_code:        r.reason_code,
          reason_name:        r.reason_name,
          reason_name_en:     r.reason_name_en,
          legal_basis:        r.legal_basis,
          ubl_category_code:  r.ubl_category_code,
          ubl_exemption_code: r.ubl_exemption_code,
        }
      },
    })
  end
end
