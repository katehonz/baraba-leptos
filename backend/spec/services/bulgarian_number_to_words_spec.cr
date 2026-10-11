require "../spec_helper"

describe BulgarianNumberToWords do
  it "converts the design fixtures" do
    BulgarianNumberToWords.convert(0.0, "BGN").should eq("Нула лева")
    BulgarianNumberToWords.convert(0.01, "BGN").should eq("Нула лева и една стотинка")
    BulgarianNumberToWords.convert(1.0, "BGN").should eq("Един лев")
    BulgarianNumberToWords.convert(2.0, "BGN").should eq("Два лева")
    BulgarianNumberToWords.convert(21.0, "BGN").should eq("Двадесет и един лева")
    BulgarianNumberToWords.convert(100.0, "BGN").should eq("Сто лева")
    BulgarianNumberToWords.convert(101.0, "BGN").should eq("Сто и един лева")
    BulgarianNumberToWords.convert(1000.0, "BGN").should eq("Хиляда лева")
    BulgarianNumberToWords.convert(1001.0, "BGN").should eq("Хиляда и един лева")
    BulgarianNumberToWords.convert(1_000_000.0, "BGN").should eq("Един милион лева")
    BulgarianNumberToWords.convert(120.0, "EUR").should eq("Сто и двадесет евро")
    BulgarianNumberToWords.convert(120.0, "BGN").should eq("Сто и двадесет лева")
    BulgarianNumberToWords.convert(-120.0, "BGN").should eq("Минус сто и двадесет лева")
    BulgarianNumberToWords.convert(1.005, "BGN").should eq("Един лев и една стотинка")
  end
end
