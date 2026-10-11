class Api::BankTransactions::Reallocate < ApiAction
  post "/api/companies/:company_id/bank_transactions/:bank_transaction_id/reallocate" do
    account_ids = BankAccountQuery.new.company_id(company_id).map(&.id)

    transaction = BankTransactionQuery.new
      .bank_account_id.in(account_ids)
      .find(bank_transaction_id)

    json_body = begin
      params.from_json
    rescue
      JSON::Any.new({} of String => JSON::Any)
    end

    new_account_id = params.get?("account_id").try(&.to_i64)
    unless new_account_id
      if raw = json_body["account_id"]?
        new_account_id = raw.as_i64? || raw.as_i?.try(&.to_i64) || raw.as_s?.try(&.to_i64?)
      end
    end
    unless new_account_id
      response.status_code = 400
      return json({success: false, error: "Моля изберете сметка"})
    end

    counterpart_id : Int64? = nil
    if raw = json_body["counterpart_id"]?
      counterpart_id = raw.as_i64? || raw.as_i?.try(&.to_i64) || raw.as_s?.try(&.to_i64?)
    end

    begin
      entry = BankTransactionAllocator.reallocate!(
        company_id.to_i64,
        transaction,
        new_account_id.not_nil!,
        counterpart_id
      )
      new_account = AccountQuery.new.find(new_account_id.not_nil!)
      json({
        success: true,
        data:    {
          journal_entry_id: entry.id,
          new_account_id:   new_account.id,
          new_account_code: new_account.code,
          new_account_name: new_account.name,
          message:          "Сметката е преразпределена от буфер към #{new_account.code} #{new_account.name}",
        },
      })
    rescue ex : BankTransactionAllocator::Error
      response.status_code = 400
      json({success: false, error: ex.message})
    end
  end
end
