require "../../../spec_helper"

describe Api::Counterparts::Create do
  it "strips EU prefix into EIK and rejects duplicates except 9 nines" do
    user = UserFactory.create &.is_super_admin(true)
    company = SaveCompany.create!(name: "API Контрагенти ООД", eik: "301111118")

    first = ApiClient.auth(user).exec_raw(
      Api::Counterparts::Create.with(company.id.to_s),
      {
        counterpart: {
          name:             "Italian SRL",
          eik:              "IT01234567890",
          vat_number:       "IT01234567890",
          counterpart_type: "both",
        },
      }.to_json
    )
    first.status_code.should eq(200)

    saved = CounterpartQuery.new.company_id(company.id).first
    saved.eik.should eq("01234567890")

    dup = ApiClient.auth(user).exec_raw(
      Api::Counterparts::Create.with(company.id.to_s),
      {
        counterpart: {
          name:             "Italian Clone",
          eik:              "01234567890",
          counterpart_type: "both",
        },
      }.to_json
    )
    dup.status_code.should eq(422)
    dup.body.should contain("вече съществува")

    nines1 = ApiClient.auth(user).exec_raw(
      Api::Counterparts::Create.with(company.id.to_s),
      {
        counterpart: {
          name:             "Unknown A",
          eik:              "999999999",
          counterpart_type: "customer",
        },
      }.to_json
    )
    nines2 = ApiClient.auth(user).exec_raw(
      Api::Counterparts::Create.with(company.id.to_s),
      {
        counterpart: {
          name:             "Unknown B",
          eik:              "999999999",
          counterpart_type: "customer",
        },
      }.to_json
    )
    nines1.status_code.should eq(200)
    nines2.status_code.should eq(200)
    CounterpartQuery.new.company_id(company.id).eik("999999999").select_count.should eq(2)
  end
end
