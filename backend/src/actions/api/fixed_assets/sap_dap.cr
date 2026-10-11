class Api::FixedAssets::SapDap < ApiAction
  get "/api/companies/:company_id/fixed_assets/sap_dap" do
    year = params.get?("year").try(&.to_i?) || Time.utc.year
    sap = (params.get?("plan") || "sap") != "dap"

    company = CompanyQuery.new.id(company_id).first
    categories = {} of Int64 => FixedAssetCategory
    FixedAssetCategoryQuery.new.company_id(company_id).each { |c| categories[c.id] = c }

    rows = [] of NamedTuple(
      inventory_number: String,
      name: String,
      category: String,
      service_date: String,
      useful_months: Int32,
      cost: Float64,
      accumulated_begin: Float64,
      months: Array(String),
      year_total: Float64,
      accumulated_end: Float64,
      remaining: Float64,
      rate: Float64,
      note: String
    )

    FixedAssetQuery.new.company_id(company_id).each do |asset|
      next if asset.status == "disposed" && (asset.disposed_at.try(&.year) || year) < year

      engine_params = DepreciationEngine.for_asset(asset, sap)
      next if engine_params.rate <= 0.0

      year_rows = DepreciationEngine.year_months(engine_params, year)
      next if year_rows.empty?

      all_to_prev = DepreciationEngine.schedule(engine_params, year - 1, 12)
      acc_begin = all_to_prev.last?.try(&.accumulated) || 0.0
      months = Array.new(12, "0.00")
      year_total = 0.0
      year_rows.each do |r|
        if r.conserved
          months[r.month - 1] = "консервация"
        elsif r.amount > 0.0
          months[r.month - 1] = "%.2f" % r.amount
          year_total += r.amount
        end
      end
      acc_end = year_rows.last?.try(&.accumulated) || acc_begin
      remaining = year_rows.last?.try(&.remaining) || 0.0
      cat_name = asset.category_id.try { |id| categories[id]?.try(&.name) } || ""
      notes = [] of String
      if asset.conserved_from
        notes << "консервация #{asset.conserved_from.try(&.to_s("%d.%m.%Y"))}"
      end
      if asset.disposed_at
        notes << "отписване #{asset.disposed_at.try(&.to_s("%d.%m.%Y"))}"
      end

      rows << {
        inventory_number:   asset.inventory_number,
        name:               asset.name,
        category:           cat_name,
        service_date:       (asset.put_into_service_date || asset.acquisition_date).to_s("%d.%m.%Y"),
        useful_months:      DepreciationEngine.useful_life_months(engine_params.rate),
        cost:               engine_params.cost,
        accumulated_begin:  acc_begin.round(2),
        months:             months,
        year_total:         year_total.round(2),
        accumulated_end:    acc_end.round(2),
        remaining:          remaining.round(2),
        rate:               engine_params.rate,
        note:               notes.join("; "),
      }
    end

    json({
      success: true,
      data:    {
        company: {name: company.name, eik: company.eik},
        year:    year,
        plan:    sap ? "sap" : "dap",
        title:   sap ? "Счетоводен амортизационен план" : "Данъчнопризнат амортизационен план",
        rows:    rows,
      },
    })
  end
end
