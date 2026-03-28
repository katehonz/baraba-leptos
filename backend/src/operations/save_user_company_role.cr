class SaveUserCompanyRole < UserCompanyRole::SaveOperation
  permit_columns user_id, company_id, role_id
end
