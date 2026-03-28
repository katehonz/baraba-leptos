class Api::Counterparts::Show < ApiAction
  get "/api/companies/:company_id/counterparts/:counterpart_id" do
    counterpart = CounterpartQuery.new.company_id(company_id).id(counterpart_id).first?

    if counterpart
      json({
        success: true,
        data:    {
          id:               counterpart.id,
          name:             counterpart.name,
          vat_number:       counterpart.vat_number,
          address:          counterpart.address,
          contact_person:   counterpart.contact_person,
          email:            counterpart.email,
          phone:            counterpart.phone,
          counterpart_type: counterpart.counterpart_type,
        },
      })
    else
      response.status_code = 404
      json({success: false, error: "Counterpart not found"})
    end
  end
end
