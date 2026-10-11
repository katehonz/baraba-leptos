require "../spec_helper"

describe ControlisyParser do
  bank_xml_path = File.expand_path("../../../CONT-FILE/test_ltd.-bank_payments-01.2025_03-Банка.xml", __DIR__)
  purchases_xml_path = File.expand_path("../../../CONT-FILE/test_ltd.-pokupki-01.2025_01-Покупки.xml", __DIR__)

  it "detects a bank extract as bank even when Contractors is empty" do
    pending! "sample file missing" unless File.exists?(bank_xml_path)

    parsed = ControlisyParser.parse(File.read(bank_xml_path))
    parsed.import_type.should eq("bank")
    parsed.contractors.should be_empty
    parsed.documents.size.should be > 1
    parsed.documents.count { |d| d.vat_data }.should eq(0)
  end

  it "gives every bank row its own documentNumber while sharing ca_docId" do
    pending! "sample file missing" unless File.exists?(bank_xml_path)

    parsed = ControlisyParser.parse(File.read(bank_xml_path))
    numbers = parsed.documents.map(&.document_number)
    numbers.uniq.size.should eq(numbers.size)
    parsed.documents.map(&.ca_doc_id).uniq.size.should eq(1)
  end

  it "keeps RelDoc invoice number separate from the bank document number" do
    pending! "sample file missing" unless File.exists?(bank_xml_path)

    parsed = ControlisyParser.parse(File.read(bank_xml_path))
    with_rel = parsed.documents.select { |d| d.rel_doc }
    with_rel.should_not be_empty
    with_rel.each do |doc|
      rel = doc.rel_doc.not_nil!
      rel.document_number.should_not eq(doc.document_number)
    end
  end

  it "reads contractor EIK/VAT from RelDoc and from 401/411 lines in bank payments" do
    pending! "sample file missing" unless File.exists?(bank_xml_path)

    parsed = ControlisyParser.parse(File.read(bank_xml_path))
    payment = parsed.documents.find { |d| d.document_number == "20250106000100000001" }
    payment.should_not be_nil
    rel = payment.not_nil!.rel_doc.not_nil!
    rel.contractor_name.should eq("ТЕСТ КОНТРАГЕНТ ООД")
    rel.contractor_eik.should eq("999999999")
    rel.contractor_vat_number.should eq("BG999999999")

    credit = payment.not_nil!.accountings.first.credit_detail.not_nil!
    credit.account_number.should eq("411")
    credit.contractor_eik.should eq("999999999")
  end

  it "detects a purchase file as purchases and pairs invoice + cash payment on the same ca_docId" do
    pending! "sample file missing" unless File.exists?(purchases_xml_path)

    parsed = ControlisyParser.parse(File.read(purchases_xml_path))
    parsed.import_type.should eq("purchases")
    parsed.contractors.should_not be_empty

    invoice = parsed.documents.find { |d| d.vat_data && d.document_number == "0000000001" }
    payment = parsed.documents.find { |d| d.rel_doc && d.document_number == "0000000001" }
    invoice.should_not be_nil
    payment.should_not be_nil
    invoice.not_nil!.ca_doc_id.should eq(payment.not_nil!.ca_doc_id)
  end
end
