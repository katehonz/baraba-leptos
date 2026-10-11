require "../spec_helper"

describe BankTextNormalizer do
  it "folds POS and ПОС to the same key" do
    BankTextNormalizer.fold("POS").should eq("pos")
    BankTextNormalizer.fold("ПОС").should eq("pos")
  end

  it "matches mixed-script keywords in a bank description" do
    text = "POS трансакция - Транзакция с ДК, Покупка"
    BankMatchingService.matches_keywords?(text, "POS,ПОС,теглене").should be_true
    BankMatchingService.matches_keywords?(text, "ATM,теглене").should be_false
  end

  it "matches ATM / АТМ / теглене" do
    text = "АТМ трансакция - Теглене на пари"
    BankMatchingService.matches_keywords?(text, "ATM,АТМ,теглене,withdrawal").should be_true
  end

  it "normalizes company names ignoring legal form and case" do
    a = BankTextNormalizer.company_key("НЕВЕЛ 25 ЕООД")
    b = BankTextNormalizer.company_key("Nevel 25 EOOD")
    a.should eq(b)
  end

  it "treats hyphen and spaces the same" do
    a = BankTextNormalizer.company_key("КЗЦ-СТРОИТЕЛСТВО И МЕНИДЖМЪНТ ЕООД")
    b = BankTextNormalizer.company_key("КЗЦ СТРОИТЕЛСТВО И МЕНИДЖМЪНТ ЕООД")
    a.should eq(b)
  end

  it "matches debit direction only for outgoing amounts" do
    BankMatchingService.direction_matches?("debit", -10.0).should be_true
    BankMatchingService.direction_matches?("debit", 10.0).should be_false
    BankMatchingService.direction_matches?("credit", 10.0).should be_true
    BankMatchingService.direction_matches?("any", -1.0).should be_true
  end
end
