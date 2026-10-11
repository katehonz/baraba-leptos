class Api::Companies::Delete < ApiAction
  delete "/api/companies/:company_id" do
    user = current_user?
    company = CompanyQuery.new.id(company_id).first?

    unless company
      response.status_code = 404
      return json({success: false, message: "Фирмата не е намерена"})
    end

    unless user && (user.super_admin? || company.owner_id == user.id)
      response.status_code = 403
      return json({success: false, message: "Само собственик или супер администратор може да изтрие фирмата"})
    end

    AuditLogger.log(context, "company_delete", user.not_nil!.email, "company_id=#{company_id}")
    DeleteCompany.delete!(company)

    json({success: true, message: "Company deleted"})
  end
end
