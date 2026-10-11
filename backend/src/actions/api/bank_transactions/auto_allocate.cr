class Api::BankTransactions::AutoAllocate < ApiAction
  post "/api/companies/:company_id/bank_transactions/auto_allocate" do
    result = BankMatchingService.auto_allocate(company_id.to_i64)

    json({
      success: true,
      data:    {
        matched:          result[:matched],
        unmatched:        result[:unmatched],
        errors:           result[:errors],
        counterpart_hits: result[:counterpart_hits],
        details:          result[:details],
      },
      message: "Обработени по ключови думи: #{result[:matched]}. Без съвпадение: #{result[:unmatched]}. Грешки: #{result[:errors]}. Намерени контрагенти: #{result[:counterpart_hits]}.",
    })
  end
end
