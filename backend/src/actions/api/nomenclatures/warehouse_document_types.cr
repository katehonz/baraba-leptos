class Api::Nomenclatures::WarehouseDocumentTypes < ApiAction
  get "/api/nomenclatures/warehouse_document_types" do
    query = WarehouseDocumentTypeQuery.new.sort_order.asc_order
    unless params.get?("all") == "true"
      query = query.system_only(false)
    end
    json({
      success: true,
      data:    query.map { |r|
        {
          code:                r.code,
          name_bg:             r.name_bg,
          saft_movement_type:  r.saft_movement_type,
          needs_from:          r.needs_from,
          needs_to:            r.needs_to,
          default_is_incoming: r.default_is_incoming,
          system_only:         r.system_only,
        }
      },
    })
  end
end
