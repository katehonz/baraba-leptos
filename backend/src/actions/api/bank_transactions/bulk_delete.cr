class Api::BankTransactions::BulkDelete < ApiAction
  post "/api/companies/:company_id/bank_transactions/bulk_delete" do
    account_ids = BankAccountQuery.new.company_id(company_id).map(&.id)

    json_body = begin
      params.from_json
    rescue
      JSON::Any.new({} of String => JSON::Any)
    end

    ids = [] of Int64
    if raw = json_body["ids"]?
      raw.as_a.each do |v|
        n = v.as_i64? || v.as_i?.try(&.to_i64)
        ids << n if n
      end
    end

    if ids.empty?
      response.status_code = 400
      return json({success: false, error: "Няма избрани транзакции"})
    end

    deleted = 0
    errors = [] of String
    ids.each do |id|
      tx = BankTransactionQuery.new.bank_account_id.in(account_ids).id(id).first?
      unless tx
        errors << "##{id} не е намерена"
        next
      end
      begin
        BankTransactionDeleter.delete!(tx)
        deleted += 1
      rescue ex : BankTransactionDeleter::Error
        errors << "##{id}: #{ex.message}"
      end
    end

    json({
      success: errors.empty?,
      data:    {
        deleted: deleted,
        message: "Изтрити #{deleted} транзакции",
        errors:  errors,
      },
    })
  end
end
