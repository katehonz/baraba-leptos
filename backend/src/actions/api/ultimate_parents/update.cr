class Api::UltimateParents::Update < ApiAction
  put "/api/companies/:company_id/ultimate_parents/:id" do
    parent = UltimateParentQuery.new.id(id).company_id(company_id).first?

    if parent
      SaveUltimateParent.update(parent, params) do |operation, updated|
        if updated
          json({
            success: true,
            data:    {
              id:         updated.id,
              name_bg:    updated.name_bg,
              name_latin: updated.name_latin,
              uic:        updated.uic,
              country:    updated.country,
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
