class Api::Companies::Create < ApiAction
  post "/api/companies" do
    user = current_user?
    unless user
      response.status_code = 401
      return json({success: false, message: "Не сте автентикирани"})
    end

    SaveCompany.create(params, owner_id: user.id) do |operation, company|
      if company
        # Initialize accounts from SAF-T chart of accounts
        init_result = CompanyAccountsInitializer.initialize_accounts(company)

        json({
          success: true,
          data:    {
            id:         company.id,
            name:       company.name,
            vat_number: company.vat_number,
            address:    company.address,
          },
          accounts_initialized: init_result[:created],
          message:              init_result[:message],
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
