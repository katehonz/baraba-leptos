require "../spec_helper"

describe PaymentMethodDetector do
  it "detects cash from typical Bulgarian wording" do
    PaymentMethodDetector.detect(nil, "Начин на плащане: в брой").method.should eq("cash")
    PaymentMethodDetector.detect(nil, "Платено в брой").method.should eq("cash")
    PaymentMethodDetector.detect("cash", nil).method.should eq("cash")
  end

  it "detects card from карта / POS / терминал variants" do
    PaymentMethodDetector.detect(nil, "Плащане с карта").method.should eq("card")
    PaymentMethodDetector.detect(nil, "на POS терминал").method.should eq("card")
    PaymentMethodDetector.detect(nil, "ПОС терминал").method.should eq("card")
    PaymentMethodDetector.detect(nil, "Visa / Mastercard").method.should eq("card")
    PaymentMethodDetector.detect(nil, "банкова карта").method.should eq("card")
    PaymentMethodDetector.detect("card", nil).method.should eq("card")
  end

  it "prefers card when both cash-like and card words appear" do
    PaymentMethodDetector.detect("cash", "Платено на ПОС терминал").method.should eq("card")
    PaymentMethodDetector.detect(nil, "Каса / карта").method.should eq("card")
  end

  it "detects bank transfer" do
    PaymentMethodDetector.detect(nil, "Банков превод по сметка").method.should eq("bank")
    PaymentMethodDetector.detect("bank", nil).method.should eq("bank")
  end

  it "returns unknown when nothing matches" do
    PaymentMethodDetector.detect(nil, "Фактура за услуги").method.should eq("unknown")
    PaymentMethodDetector.detect(nil, nil).method.should eq("unknown")
  end
end
