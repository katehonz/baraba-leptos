class Api::Counterparts::Delete < ApiAction
  delete "/api/companies/:company_id/counterparts/:counterpart_id" do
    counterpart = CounterpartQuery.new.company_id(company_id).id(counterpart_id).first
    DeleteCounterpart.delete!(counterpart)

    json({success: true, message: "Counterpart deleted"})
  end
end
