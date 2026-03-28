class Api::BeneficialOwners::Update < ApiAction
  put "/api/companies/:company_id/beneficial_owners/:id" do
    owner = BeneficialOwnerQuery.new.id(id).company_id(company_id).first?

    if owner
      SaveBeneficialOwner.update(owner, params) do |operation, updated|
        if updated
          json({
            success: true,
            data:    {
              id:                   updated.id,
              first_name_bg:        updated.first_name_bg,
              last_name_bg:         updated.last_name_bg,
              egn:                  updated.egn,
              first_name_latin:     updated.first_name_latin,
              last_name_latin:      updated.last_name_latin,
              country:              updated.country,
              ownership_percentage: updated.ownership_percentage,
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
    else
      response.status_code = 404
      json({success: false, message: "Not found"})
    end
  end
end
