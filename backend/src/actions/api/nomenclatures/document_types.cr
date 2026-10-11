class Api::Nomenclatures::DocumentTypes < ApiAction
  get "/api/nomenclatures/document_types" do
    rows = DocumentTypeCodeQuery.new.sort_order.asc_order
    json({
      success: true,
      data:    rows.map { |r|
        {
          code:       r.code,
          name:       r.name,
          name_en:    r.name_en,
          applies_to: r.applies_to,
          is_common:  r.is_common,
        }
      },
    })
  end
end
