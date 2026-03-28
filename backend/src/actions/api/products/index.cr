class Api::Products::Index < ApiAction
  get "/api/companies/:company_id/products" do
    products = ProductQuery.new.company_id(company_id)

    # Filter by active status if provided
    if active = params.get?("active")
      products = products.is_active(active == "true")
    end

    json({
      success: true,
      data:    products.map { |p| product_json(p) },
    })
  end

  private def product_json(p : Product)
    {
      id:                   p.id,
      code:                 p.code,
      name:                 p.name,
      description:          p.description,
      product_type:         p.product_type,
      price:                p.price,
      measure_unit:         p.measure_unit,
      commodity_code:       p.commodity_code,
      tax_type:             p.tax_type,
      tax_code:             p.tax_code,
      is_active:            p.is_active,
      inventory_account_id: p.inventory_account_id,
      revenue_account_id:   p.revenue_account_id,
    }
  end
end
