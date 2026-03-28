class Api::UltimateParents::Index < ApiAction
  get "/api/companies/:company_id/ultimate_parents" do
    parents = UltimateParentQuery.new
      .company_id(company_id)
      .is_active(true)

    json({
      success: true,
      data:    parents.map { |p| serialize(p) },
    })
  end

  private def serialize(parent : UltimateParent)
    {
      id:         parent.id,
      name_bg:    parent.name_bg,
      name_latin: parent.name_latin,
      uic:        parent.uic,
      country:    parent.country,
    }
  end
end
