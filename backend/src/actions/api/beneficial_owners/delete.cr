class Api::BeneficialOwners::Delete < ApiAction
  delete "/api/companies/:company_id/beneficial_owners/:id" do
    owner = BeneficialOwnerQuery.new.id(id).company_id(company_id).first?

    if owner
      # Soft delete
      SaveBeneficialOwner.update!(owner, is_active: false)
      json({success: true})
    else
      response.status_code = 404
      json({success: false, message: "Not found"})
    end
  end
end
