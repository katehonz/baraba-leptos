require "../spec_helper"

describe ControlisyImporter do
  it "strips EU letters into EIK, does not duplicate real EIK, allows 9 and 13 nines" do
    company = SaveCompany.create!(name: "Импорт ЕИК ООД", eik: "301111117")

    xml = <<-XML
    <?xml version="1.0" encoding="utf-8"?>
    <ExportedData>
      <Contractors>
        <Contractor contractorName="Italian SRL" contractorEIK="IT01234567890" contractorVATNumber="IT01234567890" ca_contractorId="1"/>
        <Contractor contractorName="Italian SRL Dup" contractorEIK="01234567890" contractorVATNumber="IT01234567890" ca_contractorId="2"/>
        <Contractor contractorName="Spain SA" contractorEIK="ESA12345678" contractorVATNumber="ESA12345678" ca_contractorId="3"/>
        <Contractor contractorName="Unknown A" contractorEIK="999999999" contractorVATNumber="" ca_contractorId="4"/>
        <Contractor contractorName="Unknown B" contractorEIK="999999999" contractorVATNumber="" ca_contractorId="5"/>
        <Contractor contractorName="Unknown C" contractorEIK="9999999999999" contractorVATNumber="" ca_contractorId="6"/>
        <Contractor contractorName="Unknown D" contractorEIK="9999999999999" contractorVATNumber="" ca_contractorId="7"/>
      </Contractors>
      <Documents></Documents>
    </ExportedData>
    XML

    result = ControlisyImporter.new(company.id).import(xml)
    result.success.should be_true

    cps = CounterpartQuery.new.company_id(company.id).to_a
    cps.count { |c| c.eik == "01234567890" }.should eq(1)
    spain = cps.find { |c| c.name == "Spain SA" }
    spain.should_not be_nil
    spain.not_nil!.eik.should eq("A12345678")
    cps.count { |c| c.eik == "999999999" }.should eq(2)
    cps.count { |c| c.eik == "9999999999999" }.should eq(2)

    reimport = ControlisyImporter.new(company.id).import(xml)
    reimport.success.should be_true
    CounterpartQuery.new.company_id(company.id).eik("01234567890").select_count.should eq(1)
    CounterpartQuery.new.company_id(company.id).eik("999999999").select_count.should eq(2)
  end
end
