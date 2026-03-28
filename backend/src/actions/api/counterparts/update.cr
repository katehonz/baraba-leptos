class Api::Counterparts::Update < ApiAction
  put "/api/companies/:company_id/counterparts/:counterpart_id" do
    counterpart = CounterpartQuery.new.company_id(company_id).id(counterpart_id).first

    SaveCounterpart.update(counterpart, params) do |operation, updated|
      if operation.saved?
        json({
          success: true,
          data:    {
            id:               updated.id,
            name:             updated.name,
            vat_number:       updated.vat_number,
            address:          updated.address,
            contact_person:   updated.contact_person,
            email:            updated.email,
            phone:            updated.phone,
            counterpart_type: updated.counterpart_type,
          },
        })
      else
        response.status_code = 422
        json({
          success: false,
          errors:  operation.errors.map { |key, errors| {field: key, messages: errors} },
        })
      end
    end
  end
end
