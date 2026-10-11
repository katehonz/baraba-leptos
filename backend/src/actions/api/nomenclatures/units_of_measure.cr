class Api::Nomenclatures::UnitsOfMeasure < ApiAction
  get "/api/nomenclatures/units_of_measure" do
    query = SaftUnitOfMeasureQuery.new.sort_order.asc_order.code.asc_order
    if params.get?("common") == "true"
      query = query.is_common(true)
    end

    json({
      success: true,
      data:    query.map { |u|
        {
          code:       u.code,
          name_bg:    u.name_bg,
          name_en:    u.name_en,
          symbol:     u.symbol,
          category:   u.category,
          is_common:  u.is_common,
          sort_order: u.sort_order,
        }
      },
    })
  end
end
