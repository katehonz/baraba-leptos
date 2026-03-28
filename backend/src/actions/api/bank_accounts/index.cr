class Api::BankAccounts::Index < ApiAction
  get "/api/companies/:company_id/bank_accounts" do
    accounts = BankAccountQuery.new.company_id(company_id)

    json({
      success: true,
      data:    accounts.map { |a|
        gl_account = if gl_id = a.gl_account_id
                       AccountQuery.new.id(gl_id).first?
                     end
        buffer_account = if buf_id = a.buffer_account_id
                           AccountQuery.new.id(buf_id).first?
                         end

        {
          id:                   a.id,
          name:                 a.name,
          iban:                 a.iban,
          bic:                  a.bic,
          currency:             a.currency,
          bank_name:            a.bank_name,
          integration_type:     a.integration_type,
          gl_account_id:        a.gl_account_id,
          gl_account_code:      gl_account.try(&.code),
          gl_account_name:      gl_account.try(&.name),
          buffer_account_id:    a.buffer_account_id,
          buffer_account_code:  buffer_account.try(&.code),
          buffer_account_name:  buffer_account.try(&.name),
        }
      },
    })
  end
end
