class Api::UltimateParents::Delete < ApiAction
  delete "/api/companies/:company_id/ultimate_parents/:id" do
    parent = UltimateParentQuery.new.id(id).company_id(company_id).first?

    if parent
      # Soft delete
      SaveUltimateParent.update!(parent, is_active: false)
      json({success: true})
    else
      response.status_code = 404
      json({success: false, message: "Not found"})
    end
  end
end
