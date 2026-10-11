class Api::Counterparts::Index < ApiAction
  get "/api/companies/:company_id/counterparts" do
    counterparts = CounterpartQuery.new.company_id(company_id)

    # Filter by type if provided
    if type = params.get?("type")
      counterparts = counterparts.counterpart_type(type.to_s)
    end

    json({
      success: true,
      data:    counterparts.map { |c| counterpart_json(c) },
    })
  end

  private def counterpart_json(c : Counterpart)
    {
      id:               c.id,
      name:             c.name,
      eik:              c.eik,
      vat_number:       c.vat_number,
      address:          c.address,
      city:             c.city,
      country_code:     c.country_code,
      contact_person:   c.contact_person,
      email:            c.email,
      phone:            c.phone,
      counterpart_type: c.counterpart_type,
      is_active:        c.is_active,
    }
  end
end
