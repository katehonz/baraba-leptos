class Api::BeneficialOwners::Index < ApiAction
  get "/api/companies/:company_id/beneficial_owners" do
    owners = BeneficialOwnerQuery.new
      .company_id(company_id)
      .is_active(true)

    json({
      success: true,
      data:    owners.map { |o| serialize(o) },
    })
  end

  private def serialize(owner : BeneficialOwner)
    {
      id:                   owner.id,
      first_name_bg:        owner.first_name_bg,
      last_name_bg:         owner.last_name_bg,
      egn:                  owner.egn,
      first_name_latin:     owner.first_name_latin,
      last_name_latin:      owner.last_name_latin,
      country:              owner.country,
      ownership_percentage: owner.ownership_percentage,
    }
  end
end
