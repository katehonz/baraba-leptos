require "../spec_helper"

describe Counterpart do
  it "strips EU VAT letters from EIK" do
    Counterpart.strip_eu_prefix("BG123456789").should eq("123456789")
    Counterpart.strip_eu_prefix("IT 01234567890").should eq("01234567890")
    Counterpart.strip_eu_prefix("es-a12345678").should eq("A12345678")
    Counterpart.strip_eu_prefix("123456789").should eq("123456789")
    Counterpart.normalize_eik(" BG-123456789 ").should eq("123456789")
    Counterpart.normalize_eik("").should be_nil
    Counterpart.generic_eik?("999999999").should be_true
    Counterpart.generic_eik?("BG999999999").should be_true
    Counterpart.generic_eik?("9999999999999").should be_true
    Counterpart.generic_eik?("123456789").should be_false
  end
end

describe SaveCounterpart do
  it "stores EIK without BG/IT/ES prefix and fills EIK from VAT when blank" do
    company = SaveCompany.create!(name: "ЕИК Норм ООД", eik: "301111111")

    italian = SaveCounterpart.create!(
      company_id: company.id,
      name: "Italian SRL",
      eik: "IT01234567890",
      vat_number: "IT01234567890",
      counterpart_type: "both"
    )
    italian.eik.should eq("01234567890")
    italian.vat_number.should eq("IT01234567890")

    from_vat = SaveCounterpart.create!(
      company_id: company.id,
      name: "Spain SA",
      vat_number: "ESA12345678",
      counterpart_type: "customer"
    )
    from_vat.eik.should eq("A12345678")
  end

  it "rejects a duplicate EIK in the same company" do
    company = SaveCompany.create!(name: "Дубл ООД", eik: "301111112")
    SaveCounterpart.create!(
      company_id: company.id,
      name: "Първи ООД",
      eik: "123456789",
      counterpart_type: "both"
    )

    SaveCounterpart.create(
      company_id: company.id,
      name: "Втори ООД",
      eik: "BG123456789",
      counterpart_type: "both"
    ) do |op, saved|
      saved.should be_nil
      op.eik.errors.join.should contain("вече съществува")
      op.eik.errors.join.should contain("Първи ООД")
    end
  end

  it "allows the same EIK in another company" do
    a = SaveCompany.create!(name: "Фирма А ООД", eik: "301111113")
    b = SaveCompany.create!(name: "Фирма Б ООД", eik: "301111114")
    SaveCounterpart.create!(company_id: a.id, name: "Клиент А", eik: "555666777", counterpart_type: "both")
    other = SaveCounterpart.create!(company_id: b.id, name: "Клиент Б", eik: "555666777", counterpart_type: "both")
    other.eik.should eq("555666777")
  end

  it "allows duplicate 9 nines and 13 nines for unknown third-country parties" do
    company = SaveCompany.create!(name: "Износ ООД", eik: "301111115")
    SaveCounterpart.create!(company_id: company.id, name: "Unknown A", eik: "999999999", counterpart_type: "customer")
    SaveCounterpart.create!(company_id: company.id, name: "Unknown B", eik: "999999999", counterpart_type: "customer")
    SaveCounterpart.create!(company_id: company.id, name: "Unknown C", eik: "9999999999999", counterpart_type: "customer")
    SaveCounterpart.create!(company_id: company.id, name: "Unknown D", eik: "9999999999999", counterpart_type: "customer")

    CounterpartQuery.new.company_id(company.id).eik("999999999").select_count.should eq(2)
    CounterpartQuery.new.company_id(company.id).eik("9999999999999").select_count.should eq(2)
  end

  it "allows updating a counterpart without treating its own EIK as a duplicate" do
    company = SaveCompany.create!(name: "Редакция ООД", eik: "301111116")
    cp = SaveCounterpart.create!(company_id: company.id, name: "Стар", eik: "888777666", counterpart_type: "both")
    updated = SaveCounterpart.update!(cp, name: "Нов")
    updated.eik.should eq("888777666")
    updated.name.should eq("Нов")
  end
end
